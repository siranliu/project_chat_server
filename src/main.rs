Hi !


use std::net::{TcpListener};
use std::collections::HashMap;
use std::io::{BufReader,BufWriter};
use std::net::TcpStream;
use std::sync::{Arc,Condvar,Mutex};
use std::io::prelude::*;
use std::thread;
use std::sync::mpsc::channel;
use std::sync::mpsc::{sync_channel, SyncSender  , Receiver};



extern crate chan;
//this is a rudimentary chat server where users are allowed to create and join chat rooms
//many bugs are yet to be fixed

fn main() {
   	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
   	let mut group_chat = Group_chat :: new() ;
    let group_chat = Arc :: new(Mutex::new(group_chat));

    for stream in listener.incoming() {
        let group_chat = group_chat.clone() ;
        match stream {
	        Ok(stream) => {
                thread::spawn(move|| {
                    user_loop(stream , group_chat);
	            });
	        }
	        Err(e) => {}
	    }
	} 
}

fn user_loop (mut stream : TcpStream  ,group_chat : Arc<Mutex<Group_chat>>){
    let mut stream_create = stream.try_clone().unwrap() ;
    let mut stream_create_join = stream_create.try_clone().unwrap() ;
    let mut stream_join = stream_create.try_clone().unwrap() ;
    let mut stream_join_join = stream_create.try_clone().unwrap() ;
    stream.write("Join or Creat group chat? Enter 1 for Join or 2 for Create : ".as_bytes());
    let mut read_method = BufReader::new(stream) ;
    let mut my_string = String :: new() ;
    read_method.read_line(&mut my_string) ;
    let mut vec : Vec<char> = Vec :: new() ;
    for x in my_string.clone().chars() {
        vec.push(x);
    }
    
    if(vec[0] == '2'){
        {
            stream_create.write("please enter chatroom name:".as_bytes());
            let mut read_method = BufReader::new(stream_create) ;
            let mut my_string = String :: new() ;
            read_method.read_line(&mut my_string) ;
            let group_chat1 = group_chat.clone();  
            create_chatroom(group_chat1 , my_string.clone());
            let group_chat2 = group_chat.clone();
            join_group_chat(stream_create_join , my_string.clone() , group_chat2);
        }   
    }
    else if (vec[0] == '1'){
        let group_chat3 = group_chat.clone();
        let vec = group_chat3.lock().unwrap().get_chatroom_list();
        for x in vec {
            stream_join.write(x.as_bytes());
            stream_join.write("\n".as_bytes());
        }
        stream_join.write("please enther chatroom name:".as_bytes());
        let mut read_method = BufReader::new(stream_create) ;
        let mut my_string = String :: new() ;
        read_method.read_line(&mut my_string) ;
        let group_chat = group_chat.clone();
        join_group_chat(stream_join_join , my_string.clone() , group_chat);

    }

}

fn join_group_chat (mut stream : TcpStream , name : String , group_chat :Arc<Mutex<Group_chat>>){
    let sender = group_chat.lock().unwrap().get_sender(name.clone()) ; 
    let receiver = group_chat.lock().unwrap().get_receiver(name.clone());
    let(unique_s , unique_r) = chan :: sync(100);
    group_chat.lock().unwrap().add_member(name.clone() ,unique_s );
    handle_client( stream , sender , unique_r);
}

fn handle_client(mut stream : TcpStream ,sender : chan :: Sender<String> , receiver : chan :: Receiver<String> ) {
        let mut clone_stream = stream.try_clone().unwrap() ;
        let mut clone_stream2 = stream.try_clone().unwrap() ;
        let sender = sender.clone() ;
        let receiver = receiver.clone() ;


        thread:: spawn(move || {
            loop{
                clone_stream.write(receiver.recv().unwrap().as_bytes());
            }
        });

        thread:: spawn(move || {
            loop {
                let mut read_method = BufReader::new(&clone_stream2) ;
                let mut my_string = String :: new() ;
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


pub struct channels {
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