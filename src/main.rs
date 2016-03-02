
use std::net::{TcpListener};
use std::collections::HashMap;
use std::io::{BufReader,BufWriter};
use std::net::TcpStream;
use std::sync::{Arc,Condvar,Mutex};
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::{sync_channel, SyncSender  , Receiver};
use std::fs::File;
use std::fs::OpenOptions;


extern crate chan;

fn main() {
   	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
   	let mut group_chat = Group_chat :: new() ;
    let mut users = User_info_map :: new() ;
    let users = Arc :: new(Mutex::new(users));
    let group_chat = Arc :: new(Mutex::new(group_chat));


    for stream in listener.incoming() {
        let group_chat = group_chat.clone() ;
        let users = users.clone();
        match stream {
	        Ok(stream) => {
                thread::spawn(move|| {
                    login(stream , group_chat , users);
	            });
	        }
	        Err(e) => {}
	    }
	} 
}

fn login(mut stream : TcpStream , group_chat : Arc<Mutex<Group_chat>> , users : Arc<Mutex<User_info_map>>){
    let mut stream_loop = stream.try_clone().unwrap() ;
    let mut stream_loop2 = stream.try_clone().unwrap() ;
    let mut read_method = BufReader::new(stream) ;
    loop{
        let mut stream_loop2 = stream_loop.try_clone().unwrap() ;
        stream_loop.write("If existing user please enter Y or to create an account enter N ".as_bytes());
        let mut my_string = String :: new() ;
        read_method.read_line(&mut my_string);
        let mut vec : Vec<char> = Vec :: new() ;
        for x in my_string.clone().chars() {
            vec.push(x);
        }
        if vec[0] == 'Y'{

            stream_loop2.write("please enter your user name : ".as_bytes());
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string);
            let name = my_string.clone();
            stream_loop2.write("please enter your password : ".as_bytes());
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string);
            let password = my_string.clone();
            let mut flag = false;
            {
                flag = users.lock().unwrap().contains_user(name.clone());
            }
            if flag {
                let mut password2 = String :: new() ;
                {
                    password2 = users.lock().unwrap().get_password(name.clone());
                }
                if password2 == password{
                    let group_chat2 = group_chat.clone();
                    user_loop(stream_loop2 , group_chat2 , name.clone());
                    break ;
                }
                else{
                    stream_loop2.write("incorrect password \n".as_bytes());
                }
            }
            else{
                stream_loop2.write("no such user exists ! \n".as_bytes());
            }
            
        }
        else if vec[0] == 'N'{
            stream_loop2.write("please enter your user name : ".as_bytes());
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string);
            let name = my_string.clone();
            stream_loop2.write("please enter your password : ".as_bytes());
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string);
            let password = my_string.clone();
            users.lock().unwrap().Create_user(name.clone() , password.clone());


        }
        else{
            stream_loop2.write("Please give valid response \n".as_bytes());
        }



    }

}

fn user_loop (mut stream : TcpStream  ,group_chat : Arc<Mutex<Group_chat>> , name : String ){
    let mut stream = stream.try_clone().unwrap();
    loop{
        let mut stream_loop = stream.try_clone().unwrap();
        let mut stream_loop2 = stream.try_clone().unwrap();
        stream_loop.write("Join or Creat group chat? Enter J for Join or C for Create : ".as_bytes());
        let mut read_method = BufReader::new(stream_loop) ;
        let mut my_string = String :: new() ;
        read_method.read_line(&mut my_string) ;
        let mut vec : Vec<char> = Vec :: new() ;
        for x in my_string.clone().chars() {
            vec.push(x);
        }
        if(vec[0] == 'C'){
            {
                stream_loop2.write("please enter chatroom name:".as_bytes());
                let mut my_string = String :: new() ;
                read_method.read_line(&mut my_string) ;
                let group_chat1 = group_chat.clone();  
                create_chatroom(group_chat1 , my_string.clone());
                let group_chat2 = group_chat.clone();
                join_group_chat(stream_loop2 , my_string.clone() , group_chat2 , name.clone());
                break;
            }   
        }
        else if (vec[0] == 'J'){
            let group_chat3 = group_chat.clone();
            let vec = group_chat3.lock().unwrap().get_chatroom_list();
            if vec.len() == 0 {
                stream_loop2.write("There are no live chatrooms\n".as_bytes());
                continue;
            }
            for x in vec {
                stream_loop2.write(x.as_bytes());
                stream_loop2.write("\n".as_bytes());
            }
            stream_loop2.write("please enther chatroom name:".as_bytes());
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string) ;
            let group_chat = group_chat.clone();
            join_group_chat(stream_loop2 , my_string.clone() , group_chat , name.clone());
            break;

        }
        else{
            stream_loop2.write("please enter valid response\n".as_bytes());
            continue;
        }
    }
    

}

fn join_group_chat (mut stream : TcpStream , name : String , group_chat :Arc<Mutex<Group_chat>> , user_name : String){
    let sender = group_chat.lock().unwrap().get_sender(name.clone()) ; 
    let receiver = group_chat.lock().unwrap().get_receiver(name.clone());
    let(unique_s , unique_r) = chan :: sync(100);
    group_chat.lock().unwrap().add_member(name.clone() ,unique_s );
    handle_client( stream , sender , unique_r , user_name.clone());
}

fn handle_client(mut stream : TcpStream ,sender : chan :: Sender<String> , receiver : chan :: Receiver<String> , mut name : String) {
        let mut clone_stream = stream.try_clone().unwrap() ;
        let mut clone_stream2 = stream.try_clone().unwrap() ;
        let sender = sender.clone() ;
        let receiver = receiver.clone() ;

        name.pop();
        name.pop();

        let mut name_2 = name.clone();

        thread:: spawn(move || {
            loop{

                let rec_message = receiver.recv().unwrap();
                let mut rec_message_temp = rec_message.clone();

                rec_message_temp.truncate(name.len());

                if  rec_message_temp != name.clone() {                    
                        clone_stream.write(rec_message.as_bytes());
                }
            }
        });

        thread:: spawn(move || {
            loop {
                let mut read_method = BufReader::new(&clone_stream2) ;
                let mut my_string = String :: new() ;
                my_string = name_2.clone() + " : "+ &my_string;
                read_method.read_line(&mut my_string) ;
                sender.send(my_string); 
            }
        });



}


fn create_chatroom(group_chat : Arc<Mutex<Group_chat>> , name : String) {
    let (sender , receiver) = chan :: sync(100000);
    {
        group_chat.lock().unwrap().create_group(name.clone() , sender.clone() , receiver.clone());
    }
    thread :: spawn(move || {
        loop {
            let line : String = receiver.recv().unwrap() ;
            let list_sender = group_chat.lock().unwrap().get_sender_list(name.clone());
            for x in 0..list_sender.len(){
                list_sender[x].send(line.clone());
            }
        }

    });
}

struct User_info{
    name : String ,
    password : String,
    friend_list : Vec<String>,

}

struct User_info_map{
    map : HashMap<String , User_info>
}

impl User_info_map{
    fn new()->User_info_map{
        let mut map : HashMap<String , User_info> = HashMap :: new() ;
        let mut f = File::open("User_info.txt").unwrap() ;
        let mut s = String::new();
        f.read_to_string(&mut s);
        let mut split = s.split("////////");
        for temp in split {
            let mut split2 = temp.split(" ");
            let vec: Vec<&str> = split2.collect() ;
            let mut vec_frds : Vec<String> = Vec :: new() ;
            if vec.len() > 2 {
                for x in 2..vec.len(){
                    vec_frds.push(vec[x].to_string().clone());
                }
            }
            let mut user = User_info{name:vec[0].to_string().clone() , password : vec[1].to_string().clone() , friend_list:vec_frds};
            map.insert(vec[0].to_string().clone() , user);
        }
        User_info_map{
            map : map
        }

    }

    fn get_users(&mut self) -> Vec<String>{
        let mut vec : Vec<String> = Vec :: new() ;
        for key in self.map.keys(){
            vec.push(key.clone());
        }
        return vec ;
    }

    fn contains_user(&mut self , name : String) -> bool{
        return self.map.contains_key(& name);
    }

    fn get_password(&mut self , name : String) -> String {
        self.map.get(&name).unwrap().password.clone()
    }

    fn Create_user(&mut self , name : String , password : String) {
        let mut options = OpenOptions::new();
        options.write(true).append(true);
        let file = match options.open("User_info.txt") {
                    Ok(file) => file,
                    Err(..) => panic!("wth"),
        };
        let mut writer = BufWriter::new(&file);
        writer.write("////////".to_string().as_bytes());
        writer.write(name.as_bytes());
        writer.write(" ".to_string().as_bytes());
        writer.write(password.as_bytes());
        let temp = User_info{name : name.clone() , password : password.clone() , friend_list : Vec::new()} ;
        self.map.insert(name , temp);
    }
    fn add_friend(&mut self , name : String , friend : String){
        let mut options = OpenOptions::new();
        options.write(true).append(true);
        let file = match options.open("User_info.txt") {
                    Ok(file) => file,
                    Err(..) => panic!("wth"),
        };
        let mut writer = BufWriter::new(&file);
        writer.write("////////".to_string().as_bytes());
        writer.write(name.as_bytes());
        writer.write(" ".to_string().as_bytes());
        writer.write(self.map.get(&name).unwrap().password.clone().as_bytes());
        for x in 0..self.map.get(&name).unwrap().friend_list.len(){
            writer.write(" ".to_string().as_bytes());
            writer.write(self.map.get(&name).unwrap().friend_list[x].clone().as_bytes());
        }
        writer.write(" ".to_string().as_bytes());
        writer.write(friend.as_bytes());
        self.map.get_mut(&name).unwrap().friend_list.push(friend);


    }

    fn get_friend_list(&mut self , name : String) ->Vec<String>{
        let mut vec : Vec<String> = Vec :: new() ;
        for x in 0..self.map.get_mut(&name).unwrap().friend_list.len(){
            vec.push(self.map.get(&name).unwrap().friend_list[x].clone())
        }
        return vec ;
    }
}





struct channels {
    sender : chan ::Sender<String> ,
    receiver : chan :: Receiver<String>,
    list_sender : Vec<chan :: Sender<String>>,
}

impl channels {
    fn add_member(&mut self ,sender: chan :: Sender<String> ) {
        self.list_sender.push(sender);
    }
}


struct Group_chat {
    map : HashMap<String , channels> 
}
impl Group_chat {
    fn new() -> Group_chat{
    	Group_chat{
    		 map : HashMap :: new() 
    	}
    }

    fn create_group(&mut self , name: String , sender : chan ::Sender<String> , receiver : chan :: Receiver<String>) {
        let mut temp = channels{sender : sender , receiver : receiver,list_sender : Vec:: new()} ;
        self.map.insert(name , temp);
    }

    fn get_sender(&mut self , name: String) -> chan::Sender<String>{
    	match self.map.get(&name) {
    	    Some(temp) => {
    	    	return temp.sender.clone() 
    	    }
    	    None => {
    	    	panic!("no such group") ;
    	    }
    	}

    }

    fn get_receiver(&mut self , name: String) -> chan::Receiver<String>{
        match self.map.get(&name) {
            Some(temp) => {
                return temp.receiver.clone() 
            }
            None => {
                panic!("no such group") ;
            }
        }

    }

    fn add_member (&mut self , name: String , sender : chan ::Sender<String>  ){
        self.map.get_mut(&name).unwrap().add_member(sender);
    }

    fn get_sender_list(&mut self , name : String) -> Vec<chan :: Sender<String>>{
        self.map.get(&name).unwrap().list_sender.clone() 
    }

    fn get_chatroom_list(&mut self) -> Vec<String>{
        let mut vec : Vec<String> = Vec :: new() ;
        for key in self.map.keys(){
            vec.push(key.clone());
        }
        return vec ;
    }



}