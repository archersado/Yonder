use std::{io::{BufRead, BufReader, Read}, os::unix::process::CommandExt, process::{Command, Stdio}, thread, time::{Duration, Instant}};

unsafe extern "C" { fn kill(pid: i32, signal: i32) -> i32; }

fn stop_group(pid:u32){ unsafe { kill(-(pid as i32),15); } }
fn alive(pid:i32)->bool{ unsafe { kill(pid,0)==0 } }

fn main(){
    let literal=Command::new("/usr/bin/printf").args(["%s","$(touch /tmp/never-created-by-yonder-spike)"]).output().unwrap();
    assert_eq!(literal.stdout,b"$(touch /tmp/never-created-by-yonder-spike)");
    assert!(!std::path::Path::new("/tmp/never-created-by-yonder-spike").exists());

    let mut tree=Command::new("/bin/sh").arg("-c").arg("sleep 30 & echo $!; wait")
        .process_group(0).stdout(Stdio::piped()).spawn().unwrap();
    let mut line=String::new(); BufReader::new(tree.stdout.take().unwrap()).read_line(&mut line).unwrap();
    let descendant:i32=line.trim().parse().unwrap(); stop_group(tree.id());
    assert!(tree.wait().unwrap().code().is_none());
    let deadline=Instant::now()+Duration::from_secs(2);
    while alive(descendant)&&Instant::now()<deadline{thread::sleep(Duration::from_millis(10));}
    assert!(!alive(descendant));

    const LIMIT:usize=64*1024;
    let mut noisy=Command::new("/usr/bin/yes").process_group(0).stdout(Stdio::piped()).spawn().unwrap();
    let mut output=Vec::with_capacity(LIMIT); let mut chunk=[0u8;4096]; let mut truncated=false;
    loop{let count=noisy.stdout.as_mut().unwrap().read(&mut chunk).unwrap();if count==0{break}let remaining=LIMIT-output.len();output.extend_from_slice(&chunk[..count.min(remaining)]);if count>remaining||output.len()==LIMIT{truncated=true;stop_group(noisy.id());break}}
    let _=noisy.wait(); assert_eq!(output.len(),LIMIT); assert!(truncated);
    println!("{{\"literal_args\":true,\"tree_stopped\":true,\"output_bytes\":{LIMIT},\"truncated\":true,\"passed\":true}}");
}
