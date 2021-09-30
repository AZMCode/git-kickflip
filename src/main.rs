#![feature(exit_status_error)]
use failure_derive::Fail;
use strum_macros::Display;
use failure::Error;
use structopt::StructOpt;
use rand::seq::SliceRandom;
use std::fs;

#[derive(Debug, Display, Fail)]
enum KickflipError {
    UnexpectedEmptyBranchVec,
    NotInBranch
}

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short="s",long,default_value="16")]
    levels_start: u8,
    #[structopt(short="m",long,default_value="32")]
    levels_middle: u8,
    #[structopt(short,long)]
    branch: Option<String>
}

fn gen_branch_name<T: rand::Rng>(rng: &mut T) -> String {
    (0..16).into_iter().fold("kickflip_".to_string(), |prev,_| prev + &rng.gen_range(0..=9).to_string())
}

fn gen_kickflip_file<T: rand::Rng>(rng: &mut T) -> Result<(),Error> {
    let content = (0..64).into_iter().fold("Kickflipping: ".to_string(), |prev,_| prev + &rng.gen_range(0..=9).to_string());
    fs::write(".git-kickflip", content.as_bytes())?;
    Ok(())
}

fn rm_kickflip_file() -> Result<(),Error> {
    fs::remove_file(".git-kickflip").unwrap_or(());
    Ok(())
}

fn branch_off<T: rand::Rng>(from: &str, to: &str, rng: &mut T) -> Result<(),Error> {
    execute::command!("git checkout").arg(from).spawn()?.wait()?.exit_ok()?;
    execute::command!("git checkout -b").arg(to).spawn()?.wait()?.exit_ok()?;
    gen_kickflip_file(rng)?;
    execute::command!("git add .git-kickflip").spawn()?.wait()?.exit_ok()?;
    execute::command!("git commit -m \"Kickflip!\"").spawn()?.wait()?.exit_ok()?;
    Ok(())
}

fn merge_into(from: &str, to: &str) -> Result<(),Error> {
    execute::command!("git checkout").arg(to).spawn()?.wait()?.exit_ok()?;
    execute::command!("git merge --no-ff -m \"Kickflip!\" --strategy-option=theirs").arg(from).spawn()?.wait()?.exit_ok()?;
    Ok(())
}

fn get_current_branch() -> Result<String,Error> {
    let mut git_command = execute::command!("git branch --show-current");
    let command_output = git_command.output()?;
    let command_stdout_pre = String::from_utf8(command_output.stdout)?;
    let command_stdout = command_stdout_pre.trim();
    if command_stdout.len() == 0 {
        return Err(Error::from(KickflipError::NotInBranch))
    }
    Ok(command_stdout.to_string())
}
fn is_in_branch() -> Result<(),Error> {
    get_current_branch()?;
    Ok(())
}

fn split<T: rand::Rng>(branches: &mut Vec<String>, rng: &mut T) -> Result<(),Error> {
    let acting_branch = branches.choose(rng).ok_or(KickflipError::UnexpectedEmptyBranchVec)?;
    let new_branch_name = gen_branch_name(rng);
    branch_off(&acting_branch, &new_branch_name, rng)?;
    branches.push(new_branch_name);
    Ok(())
}

fn join_nonpermanent<T: rand::Rng>(branches: &mut Vec<String>, rng: &mut T) -> Result<(),Error> {
    let receiving_branch = branches.choose(rng).ok_or(KickflipError::UnexpectedEmptyBranchVec)?;
    let guest_branch = branches.choose(rng).ok_or(KickflipError::UnexpectedEmptyBranchVec)?;
    merge_into(&receiving_branch, &guest_branch)?;
    Ok(())
}

fn join_permanent<T: rand::Rng>(branches: &mut Vec<String>, rng: &mut T) -> Result<(),Error> {
    let receiving_branch = branches.choose(rng).ok_or(KickflipError::UnexpectedEmptyBranchVec)?;
    let guest_branch = branches.choose(rng).ok_or(KickflipError::UnexpectedEmptyBranchVec)?;
    merge_into(&receiving_branch, &guest_branch)?;
    *branches = branches.iter().filter(|elm| &guest_branch != elm).map(|elm| elm.clone()).collect();
    Ok(())
}

fn main() -> Result<(),Error> {
    let mut thread_rng = rand::thread_rng();
    let opts = Args::from_args();
    let mut branches: Vec<String> = vec![];
    is_in_branch()?;
    let current_branch = match opts.branch {
        Some(val) => val,
        None => get_current_branch()?
    };
    branches.push(current_branch.clone());
    for _ in 0..opts.levels_start {
        split(&mut branches, &mut thread_rng)?;
    }
    for _ in 0..opts.levels_middle {
        join_nonpermanent(&mut branches, &mut thread_rng)?;
    }
    while branches.len() > 1 {
        join_permanent(&mut branches, &mut thread_rng)?;
    }
    let last_branch = branches.pop().ok_or(KickflipError::UnexpectedEmptyBranchVec)?;
    merge_into(&last_branch,&current_branch)?;
    rm_kickflip_file()?;
    execute::command!("git add .git-kickflip").spawn()?.wait()?.exit_ok()?;
    execute::command!("git commit -m \"End of Kickflip\"").spawn()?.wait()?.exit_ok()?;
    eprintln!("Done!");
    Ok(())
}
