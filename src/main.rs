use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, thread};
use clap::Parser;
use sha2::{Digest, Sha256};
use num_cpus;

const CHUNK_SIZE: usize = 1_000_000;
const MEMORY_ORDER: Ordering = Ordering::SeqCst;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    n: u8, // сколько нулей должно быть на конце дайджеста
    #[arg(short, long)]
    f: u8, // сколько хешей нужно
}

fn main() {

    //начало текущего обрабатываемого сегмента
    let current_pos = Arc::new(AtomicUsize::new(0));

    //найдено хешей
    let found = Arc::new(AtomicUsize::new(0));

    let num_threads = num_cpus::get(); 
    let mut handlers: Vec<thread::JoinHandle<()>> = vec![]; 

    let args = Args::parse();

    for _ in 0..num_threads { // ограничиваемся числом ядер
        let target_zero_count = args.n as usize;
        let target_found = args.f as usize;
        /*
            передаем кол-во найденных и текущую позицию на числовом ряду
            в общее владение с подсчетом ссылок
         */
        let current_pos = Arc::clone(&current_pos);
        let found = Arc::clone(&found);
        
        let handle = thread::spawn(move || {
            //Будем порождать потоки, пока не найдем нужное кол-во хешей
            while found.load(MEMORY_ORDER) < target_found {
                find_target_hash(&current_pos, &found, target_zero_count, target_found);
            }
        });
        
        handlers.push(handle);
    }
    
    for handle in handlers {
        handle.join().unwrap();
    }
}

fn find_target_hash(start_pos: &AtomicUsize, counter: &AtomicUsize, target_zero_count: usize, target_found : usize) {

    let start = start_pos.fetch_add(CHUNK_SIZE, MEMORY_ORDER);
    let end = start + CHUNK_SIZE;

    for num in start..end {
        //выходим раньше, если уже набрали нужное кол-во
        if counter.load(MEMORY_ORDER) >= target_found {
            break;
        }

        let mut sha = Sha256::new();
        sha.update(num.to_be_bytes());
        let digest_num = format!("{:X}", sha.finalize());
        
        if digest_num.ends_with(&"0".repeat(target_zero_count)) {
            counter.fetch_add(1, MEMORY_ORDER); //подняли счетчик найденных хешей
            println!("{}, {}", num, digest_num);
        }
    }
}