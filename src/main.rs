
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
use std::collections::HashSet;

extern crate chan;

fn main() {
   	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
   	let mut group_chat = Group_chat :: new() ;
    let mut users = User_info_map :: new() ;
    let users = Arc :: new(Mutex::new(users));
    let group_chat = Arc :: new(Mutex::new(group_chat));


    let mut online_users = HashMap::new();
    let online_users = Arc :: new(Mutex::new(online_users));


    for stream in listener.incoming() {
        let group_chat = group_chat.clone() ;
        let users = users.clone();
        let online_users = online_users.clone();
        match stream {
	        Ok(stream) => {
                thread::spawn(move|| {
                    login(stream , group_chat , users, online_users);
	            });
	        }
	        Err(e) => {}
	    }
	} 
}

fn login(mut stream : TcpStream , group_chat : Arc<Mutex<Group_chat>> , users : Arc<Mutex<User_info_map>>, 
        online_users : Arc<Mutex<HashMap<String , TcpStream>>>) {

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

            stream_loop2.write("Please enter your user name : ".as_bytes());
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string);
            let name = my_string.clone();
            stream_loop2.write("Please enter your password : ".as_bytes());
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
                    user_loop(stream_loop2 , group_chat2 , name.clone(), online_users , users);
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
            stream_loop2.write("Please enter your user name : ".as_bytes());
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string);
            let name = my_string.clone();
            stream_loop2.write("Please enter your password : ".as_bytes());
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

fn user_loop (mut stream : TcpStream  ,group_chat : Arc<Mutex<Group_chat>> , name : String, 
             online_users : Arc<Mutex<HashMap<String, TcpStream>>>  , users : Arc<Mutex<User_info_map>> ){
    let mut stream = stream.try_clone().unwrap();
    let mut stream_user = stream.try_clone().unwrap();

    {
    online_users.lock().unwrap().insert(name.clone() ,stream_user);

    }

    loop{
        let mut stream_loop = stream.try_clone().unwrap();
        let mut stream_loop2 = stream.try_clone().unwrap();
        let mut stream_loop3 = stream_loop2.try_clone().unwrap();
        stream_loop.write("Enter F to chat with friend or A to add new friend\nEnter J for Join or C for Create chat rooms : \n".as_bytes());
        let mut read_method = BufReader::new(stream_loop) ;
        let mut my_string = String :: new() ;
        read_method.read_line(&mut my_string) ;
        let mut vec : Vec<char> = Vec :: new() ;
        for x in my_string.clone().chars() {
            vec.push(x);
        }
        if(vec[0] == 'A'){
            stream_loop2.write("please enter your friends user name : ".as_bytes());
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string) ;
            let mut flag = false;
            {
                flag = users.lock().unwrap().contains_user(name.clone());
            }
            if flag{
                users.lock().unwrap().add_friend(name.clone() , my_string.clone());
            }
            else{
                stream_loop2.write("no such user id\n".as_bytes());
            }


        }
        else if (vec[0] == 'F'){
            let vec = users.lock().unwrap().get_friend_list(name.clone());
            let mut temp : HashSet<String> = HashSet :: new() ;
            for x in 0..vec.len(){
                let mut flag = false ;
                {
                    flag = online_users.lock().unwrap().contains_key(&vec[x].clone());
                }
                if flag{
                    temp.insert(vec[x].clone());
                }

            }
            if temp.len() == 0 {
                stream_loop2.write("no friends online bohooooo\n".as_bytes());
            }
            else{
                stream_loop2.write("Here is your online friends list : \n".as_bytes());
            /*    for x in 0..temp.len(){
                    stream_loop2.write(temp[x].clone().as_bytes());
                }*/

                for x in temp.iter() {
                    stream_loop2.write(x.clone().as_bytes());
                }

                stream_loop2.write("Enter a friends name : ".as_bytes());
                let mut my_string = String :: new() ;
                read_method.read_line(&mut my_string) ;

                if temp.contains(&my_string.clone()) {
       
                    let mut frd_stream = online_users.lock().unwrap().get_mut(&my_string).unwrap().try_clone().unwrap();
                    let mut my_string = String :: new() ;
                    read_method.read_line(&mut my_string) ;
                    let mut dm_sender_name = name.clone();
                    dm_sender_name.pop();
                    dm_sender_name.pop();
                    my_string = dm_sender_name + &" send you a direct messsage : ".to_string() + &my_string;
                    frd_stream.write(my_string.clone().as_bytes());
                    continue ;
                }
                stream_loop2.write("Wrong friend name ! \n".as_bytes());
            }



        }
        else if(vec[0] == 'C'){
            {
                stream_loop2.write("please enter chatroom name:".as_bytes());

                let mut my_string = String :: new() ;
                read_method.read_line(&mut my_string) ;
                let group_chat1 = group_chat.clone();  
                create_chatroom(group_chat1 , my_string.clone());
                let group_chat2 = group_chat.clone();

                let mut quit_flag = Quit_flag::new() ;
                let quit_flag = Arc :: new(Mutex::new(quit_flag));
                let quit_flag2 = quit_flag.clone();


                join_group_chat(stream_loop2 , my_string.clone() , group_chat2 , name.clone() , quit_flag2);
                let chat_reminder : String = "Now you are in Chatroom ".to_string() + &my_string.clone();
                stream_loop3.write(chat_reminder.as_bytes());
                while quit_flag.lock().unwrap().get() == false{}
                continue ;
            }   
        }
        else if (vec[0] == 'J'){
            let group_chat3 = group_chat.clone();
            let set = group_chat3.lock().unwrap().get_chatroom_list();
            if set.len() == 0 {
                stream_loop2.write("There are no live chatrooms\n".as_bytes());
                continue;
            }
            for x in set.iter() {
                stream_loop2.write(x.as_bytes());
            }
            stream_loop2.write("please enther chatroom name:".as_bytes());
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string) ;

            if set.contains(&my_string.clone()) {

                let group_chat = group_chat.clone();

                let mut quit_flag = Quit_flag::new() ;
                let quit_flag = Arc :: new(Mutex::new(quit_flag));
                let quit_flag2 = quit_flag.clone();


                join_group_chat(stream_loop2 , my_string.clone() , group_chat , name.clone() , quit_flag2);
                let chat_reminder : String = "Now you are in Chatroom ".to_string() + &my_string.clone();
                stream_loop3.write(chat_reminder.as_bytes());
                while quit_flag.lock().unwrap().get() == false{}
                continue ;
            }
            else {
                stream_loop2.write("Wrong chatroom name! \n".as_bytes());
            }
        }
        else{
            stream_loop2.write("please enter valid response\n".as_bytes());
            continue;
        }
    }
    

}

fn join_group_chat (mut stream : TcpStream , name : String , 
                    group_chat :Arc<Mutex<Group_chat>> , user_name : String , quit_flag : Arc<Mutex<Quit_flag>>){
    let sender = group_chat.lock().unwrap().get_sender(name.clone()) ; 
    let receiver = group_chat.lock().unwrap().get_receiver(name.clone());
    let(unique_s , unique_r) = chan :: sync(100);
    group_chat.lock().unwrap().add_member(name.clone() ,user_name.clone() , unique_s );
    handle_client( stream , sender , unique_r , user_name.clone() , quit_flag , group_chat , name.clone());
}


fn handle_client(mut stream : TcpStream ,sender : chan :: Sender<String> , 
                receiver : chan :: Receiver<String> , mut name : String , 
                quit_flag : Arc<Mutex<Quit_flag>> , group_chat : Arc<Mutex<Group_chat>> , chat_name : String) {

        let mut clone_stream = stream.try_clone().unwrap() ;
        let mut clone_stream2 = stream.try_clone().unwrap() ;
        let sender = sender.clone() ;
        let receiver = receiver.clone() ;
        let name2 = name.clone();

        name.pop();
        name.pop();

        let mut name_2 = name.clone();

        let quit_flag_thread1 = quit_flag.clone();
        let quit_flag_thread2 = quit_flag.clone();
        let group_chat1 = group_chat.clone() ;

        thread:: spawn(move || {
            loop{
                if quit_flag_thread1.lock().unwrap().get(){
                    group_chat1.lock().unwrap().remove_member(chat_name.clone() , name2);
                    break;
                }
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
                read_method.read_line(&mut my_string) ;
                my_string.pop();
                my_string.pop();
                if my_string == "QUIT".to_string() {
                    quit_flag_thread2.lock().unwrap().set() ;
                    break ;
                }
                my_string = name_2.clone() + " : "+ &my_string + "\n";
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

struct Quit_flag{
    flag : bool,
}

impl Quit_flag {
    fn new()->Quit_flag{
        Quit_flag{
            flag : false,
        }
    }
    fn set(&mut self){
        self.flag = true;
    }

    fn get(&mut self)->bool{
        if self.flag {
            return true ;
        }
        else { 
            return false;
        }
    }
}

struct User_info{
    name : String ,
    password : String,
    friend_list : HashSet<String>,

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
            let mut set_frds : HashSet<String> = HashSet :: new() ;
            if vec.len() > 2 {
                for x in 2..vec.len(){
                    set_frds.insert(vec[x].to_string().clone());
                }
            }
            let mut user = User_info{name:vec[0].to_string().clone() , password : vec[1].to_string().clone() , friend_list:set_frds};
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
        let temp = User_info{name : name.clone() , password : password.clone() , friend_list : HashSet::new()} ;
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

        for x in self.map.get(&name).unwrap().friend_list.iter() {
            writer.write(" ".to_string().as_bytes());
            writer.write(x.clone().as_bytes());
        }
        writer.write(" ".to_string().as_bytes());
        writer.write(friend.as_bytes());
        self.map.get_mut(&name).unwrap().friend_list.insert(friend);


    }

    fn get_friend_list(&mut self , name : String) ->Vec<String>{
        let mut set : HashSet<String> = HashSet::new();
        let mut vec : Vec<String> = Vec :: new() ;

        for x in self.map.get(&name).unwrap().friend_list.iter() {
            vec.push(x.clone());
        }

        return vec ;
    }
}





struct channels {
    sender : chan ::Sender<String> ,
    receiver : chan :: Receiver<String>,
    list_sender : HashMap<String , chan :: Sender<String>> ,
}

impl channels {
    fn add_member(&mut self , name : String,sender: chan :: Sender<String> ) {
       self.list_sender.insert(name , sender);
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
        let mut temp = channels{sender : sender , receiver : receiver,list_sender : HashMap:: new()} ;
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

    fn add_member (&mut self , chat_name: String ,users_name : String ,sender : chan ::Sender<String>  ){
        self.map.get_mut(&chat_name).unwrap().add_member(users_name , sender);

    }

    fn get_sender_list(&mut self , name : String) -> Vec<chan :: Sender<String>>{
        let mut vec = Vec :: new();
        for key in self.map.get(&name).unwrap().list_sender.keys(){
            vec.push(self.map.get(&name).unwrap().list_sender.get(key).unwrap().clone());
        }
        vec
        //self.map.get(&name).unwrap().list_sender.clone() 
    }

    fn get_chatroom_list(&mut self) -> HashSet<String>{
        let mut set : HashSet<String> = HashSet :: new() ;
        for key in self.map.keys(){
            set.insert(key.clone());
        }
        return set ;
    }
    
    fn remove_member(&mut self , chat_room : String , name : String){
       let temp : &mut channels =  self.map.get_mut(&chat_room).unwrap();
       temp.list_sender.remove(&name);
    }
    



}