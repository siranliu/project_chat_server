#![allow(warnings)]
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
        if vec.len() != 3 {
            continue;
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
        stream_loop.write("Enter F to chat with friend or A to add new friend\nEnter J for Join or C for Create chat rooms : ".as_bytes());
        let mut read_method = BufReader::new(stream_loop) ;
        let mut my_string = String :: new() ;
        read_method.read_line(&mut my_string) ;

        my_string.pop();
        my_string.pop();
        {
            users.lock().unwrap().set_busy_true(name.clone()) ;
        }
        let mut flag = false;
        {
            flag = users.lock().unwrap().get_priavte_chat(name.clone());
        }
        if flag {


            if my_string == "Yes".to_string(){
                stream_loop2.write("you are now in private chat enter start to begin chat. \n".as_bytes());
                loop {
                    if users.lock().unwrap().get_busy(name.clone())==false {
                        break ;
                    }
                continue ;    
                }
            }
            else if my_string == "No".to_string(){
                stream_loop2.write("Decline chat request? (Y/N) \n".as_bytes());
                loop {
                    if users.lock().unwrap().get_busy(name.clone())==false {
                        break ;
                    }
                }
                continue ;  
            }
            else {
                stream_loop2.write("Invalid response , Decline chat request? (Y/N), If Enter N chat will start\n".as_bytes());
                loop {
                    if users.lock().unwrap().get_busy(name.clone())==false {
                        break ;
                    }
                }
                continue ; 
            }
        }
        

        else{

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
                    flag = users.lock().unwrap().contains_user(my_string.clone());
                }
                if flag{
                    users.lock().unwrap().add_friend(name.clone() , my_string.clone());
                    {
                        users.lock().unwrap().set_busy_false(name.clone()) ;
                    }
                }
                else{
                    stream_loop2.write("no such user id\n".as_bytes());
                    {
                        users.lock().unwrap().set_busy_false(name.clone()) ;
                    }
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
                    {
                        users.lock().unwrap().set_busy_false(name.clone()) ;
                    }
                }
                else{
                    stream_loop2.write("Here is your online friends list : \n".as_bytes());

                    for x in temp.iter() {
                        stream_loop2.write(x.clone().as_bytes());
                    }

                    stream_loop2.write("Enter a friends name : ".as_bytes());
                    let mut my_string = String :: new() ;
                    read_method.read_line(&mut my_string) ;

                    if temp.contains(&my_string.clone()) {
                        let mut temp : bool = false ;
                        {
                            temp = users.lock().unwrap().get_busy(my_string.clone());
                        }
                        if temp {
                            stream_loop2.write("User is busy , you can only send him/her a message : ".to_string().as_bytes());
                            let mut frd_stream = online_users.lock().unwrap().get_mut(&my_string).unwrap().try_clone().unwrap();
                            let mut my_string = String :: new() ;
                            read_method.read_line(&mut my_string) ;
                            let mut dm_sender_name = name.clone();
                            dm_sender_name.pop();
                            dm_sender_name.pop();
                            my_string = dm_sender_name + &" send you a messsage : ".to_string() + &my_string + 
                                        "(If you want to reply , you have to go back to looby) \n";
                            frd_stream.write(my_string.clone().as_bytes());
                                {
                                    users.lock().unwrap().set_busy_false(name.clone()) ;
                                }
                            continue ;
                        }
                        else{
                            {
                                users.lock().unwrap().set_private_chat_true(my_string.clone());
                            }
                            let mut stream1 : TcpStream ; 
                            let mut stream2 : TcpStream ; 
                            {
                            stream1 = online_users.lock().unwrap().get_mut(&my_string.clone()).unwrap().try_clone().unwrap()
                            }
                            {
                            stream2 = online_users.lock().unwrap().get_mut(&name.clone()).unwrap().try_clone().unwrap()
                            }
                            {
                                users.lock().unwrap().set_busy_true(my_string.clone());
                            }
                            let mut name1 = my_string.clone() ;
                            let mut name1_2 = my_string.clone();
                            let mut name2 = name.clone() ;
                            name1.pop();
                            name1.pop();
                            name2.pop();
                            name2.pop();
                            let mut stream1_1 = stream1.try_clone().unwrap();
                            let mut name_temp = name.clone();
                            name_temp.pop() ;
                            name_temp.pop() ;
                            let mut  my_string = "\n".to_string()+ &name_temp + " would like to chat with you , Accept by entering Yes or else No\n" ;
                            stream1.write(my_string.as_bytes());
                            my_string = "waiting for other user to accept \n".to_string();
                            stream2.write(my_string.clone().as_bytes());
                            let mut my_string = String :: new() ;
                            let mut read = BufReader :: new(stream1_1);
                            read.read_line(&mut my_string);
                            my_string.pop();
                            my_string.pop();
                            if my_string == "start".to_string() || my_string == "N".to_string(){
                                stream1.write("Chat is live\n".to_string().as_bytes());
                                let quit_flag_indi = Quit_flag :: new() ;
                                let quit_flag_indi = Arc :: new(Mutex::new(quit_flag_indi));
                                let quit_flag_indi2 = quit_flag_indi.clone();

                                let quit_flag_indi_2 = Quit_flag :: new() ;
                                let quit_flag_indi_2 = Arc :: new(Mutex::new(quit_flag_indi_2));
                                let quit_flag_indi_2_2 = quit_flag_indi_2.clone();
                                stream2.write("Accepted , you can now start chatting\n".to_string().as_bytes());
                                user_chat_loop(stream1,stream2,name1.clone(),name2.clone() , quit_flag_indi , quit_flag_indi_2);

                                while quit_flag_indi2.lock().unwrap().get() == false || quit_flag_indi_2_2.lock().unwrap().get() == false{} ;
                                {
                                    users.lock().unwrap().set_private_chat_false(name1_2.clone());
                                }
                                {
                                    users.lock().unwrap().set_busy_false(name1_2.clone());
                                }
                                continue ;
                            }
                            else {
                                if my_string != "Y".to_string() {
                                    stream1.write("Invalid response , Request Declined\n".to_string().as_bytes());
                                }
                                {
                                    users.lock().unwrap().set_private_chat_false(name1_2.clone());
                                }
                                {
                                    users.lock().unwrap().set_busy_false(name1_2.clone());
                                }
                                stream2.write("Your request is Declined\n".as_bytes())  ;
                                {
                                    users.lock().unwrap().set_busy_false(name.clone()) ;
                                }                              
                                continue ;
                            }
                            
                        }
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

                    let chat_reminder : String = "Now you are in Chatroom ".to_string() + &my_string.clone() + &",type in 'QUIT' to quit to go back to lobby\n".to_string();
                    stream_loop3.write(chat_reminder.as_bytes());
                    while quit_flag.lock().unwrap().get() == false {}
                    {
                        users.lock().unwrap().set_busy_false(name.clone()) ;
                    }
                    continue ;
                }   
            }
            else if (vec[0] == 'J'){
                let group_chat3 = group_chat.clone();
                let set = group_chat3.lock().unwrap().get_chatroom_list();
                if set.len() == 0 {
                    stream_loop2.write("There are no live chatrooms\n".as_bytes());
                    {
                        users.lock().unwrap().set_busy_false(name.clone()) ;
                    }                    
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
                    let chat_reminder : String = "Now you are in Chatroom ".to_string() + &my_string.clone() + &",type in 'QUIT' to quit to go back to lobby\n".to_string();
                    stream_loop3.write(chat_reminder.as_bytes());
                    while quit_flag.lock().unwrap().get() == false{}
                    {
                        users.lock().unwrap().set_busy_false(name.clone()) ;
                    }                    
                    continue ;
                }
                else {
                    stream_loop2.write("Wrong chatroom name! \n".as_bytes());
                    {
                        users.lock().unwrap().set_busy_false(name.clone()) ;
                    }
                    continue;
                }
            }
            else{
                stream_loop2.write("please enter valid response\n".as_bytes());
                 {
                     users.lock().unwrap().set_busy_false(name.clone()) ;
                 }
                continue;
            }
        {
            users.lock().unwrap().set_busy_false(name.clone()) ;
        }

        }
    }
    

}

fn user_chat_loop (mut stream1 : TcpStream , mut stream2 : TcpStream , name1 : String , name2 : String , 
                    quit_flag : Arc<Mutex<Quit_flag>> , quit_flag2 : Arc<Mutex<Quit_flag>>){

    let mut stream1_2 = stream1.try_clone().unwrap() ;
    let mut stream1_3 = stream1.try_clone().unwrap() ;
    let mut stream1_4 = stream1.try_clone().unwrap() ;
    let mut stream2_2 = stream2.try_clone().unwrap() ;
    let mut stream2_3 = stream2.try_clone().unwrap() ;
    let mut stream2_4 = stream2.try_clone().unwrap() ;
    let quit_flag_thread1 = quit_flag.clone();
    let quit_flag_thread2 = quit_flag2.clone();
    let quit_flag_thread1_2 = quit_flag.clone();
    let quit_flag_thread2_2 = quit_flag2.clone();

    thread :: spawn(move || {
        loop {
            let mut read_method = BufReader::new(&stream1_2) ;
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string) ;
            my_string.pop();
            my_string.pop();

            if my_string == "QUIT".to_string() {
                    quit_flag_thread2.lock().unwrap().set() ;
                    my_string = name1.clone() + " has quit plz enter any key to go back to lobby \n" ;
                    stream2_3.write(my_string.clone().as_bytes());
                    stream1_4.write("waiting for other user to quit\n".to_string().as_bytes());
                    break ;
            }
            let mut temp = false;
            {
                temp = quit_flag_thread1.lock().unwrap().get() ;
            }
            if temp {
                quit_flag_thread2.lock().unwrap().set();
                break ;
            }
            my_string = name1.clone() + " : "+ &my_string + "\n";
            stream2_3.write(my_string.clone().as_bytes());                  // write to other users
        }
    });


    thread :: spawn(move || {
        loop {
            let mut read_method = BufReader::new(&stream2_2) ;
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string) ;
            my_string.pop();
            my_string.pop();
            if my_string == "QUIT".to_string(){
                    quit_flag_thread1_2.lock().unwrap().set() ;
                    my_string = name2.clone() + " has quit plz enter any key to go back to lobby \n" ;
                    stream1_3.write(my_string.clone().as_bytes());
                    stream2_4.write("waiting for other user to quit\n".to_string().as_bytes());
                    break ;
            }
            let mut temp = false;
            {
                temp = quit_flag_thread2_2.lock().unwrap().get() ;
            }
            if temp {
                quit_flag_thread1_2.lock().unwrap().set();
                break ;
            }
            my_string = name2.clone() + " : "+ &my_string + "\n";
            stream1_3.write(my_string.clone().as_bytes());             //Recieving from other user
        }
    });
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

        thread:: spawn(move || {              // Recieving from group chat
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

        thread:: spawn(move || {                 //send to group chat
            loop {
                let mut read_method = BufReader::new(&clone_stream2) ;
                let mut my_string = String :: new() ;
                read_method.read_line(&mut my_string) ;
                my_string.pop();
                my_string.pop();
                if my_string == "QUIT".to_string() {
                    quit_flag_thread2.lock().unwrap().set() ;
                    my_string = name_2.clone() + " has left the group chat.\n";
                    sender.send(my_string); 
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
    busy : bool ,
    private_chat : bool ,

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
            let mut user = User_info{name:vec[0].to_string().clone() , password : vec[1].to_string().clone() ,
                             friend_list:set_frds , busy : false , private_chat : false};
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
        let temp = User_info{name : name.clone() , password : password.clone() , 
                    friend_list : HashSet::new() , busy : false, private_chat : false};
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
    fn set_busy_true(&mut self , user_name : String){
        self.map.get_mut(&user_name).unwrap().busy = true;
    }
    fn set_busy_false(&mut self , user_name : String){
        self.map.get_mut(&user_name).unwrap().busy = false;
    }

    fn get_busy(&mut self , user_name : String) -> bool{
        if(self.map.get(&user_name).unwrap().busy){
            return true;
         }
        else {
            return false;
        }
    }

    fn set_private_chat_true(&mut self , user_name : String){
        self.map.get_mut(&user_name).unwrap().private_chat = true;
    }
    fn set_private_chat_false(&mut self , user_name : String){
        self.map.get_mut(&user_name).unwrap().private_chat = false;
    }

    fn get_priavte_chat(&mut self , user_name : String) -> bool{
        if(self.map.get(&user_name).unwrap().private_chat){
            return true;
         }
        else {
            return false;
        }
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