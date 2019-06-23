use crate::board::Board;
use crate::move_generator::MoveGenerator;

use quanta::Clock;

use std::cmp::min;
use std::fmt;
use std::ops::{Add, AddAssign};
use std::sync::mpsc::{self, channel};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Default)]
pub struct PerftContext {
    nodes: u128,
    captures: u128,
    ep: u128,
    castles: u128,
    promotions: u128,
    checks: u128,
    disco_checks: u128,
    double_checks: u128,
    checkmates: u128,
    elapsed: u64,
}

impl PerftContext {
    pub fn new() -> PerftContext {
        PerftContext {
            nodes: 0,
            captures: 0,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            disco_checks: 0,
            double_checks: 0,
            checkmates: 0,
            elapsed: 0,
        }
    }
}

impl Add for PerftContext {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            nodes: self.nodes + other.nodes,
            captures: self.captures + other.captures,
            ep: self.ep + other.ep,
            castles: self.castles + other.castles,
            promotions: self.promotions + other.promotions,
            checks: self.checks + other.checks,
            disco_checks: self.disco_checks + other.disco_checks,
            double_checks: self.double_checks + other.double_checks,
            checkmates: self.checkmates + other.checkmates,
            elapsed: self.elapsed + other.elapsed,
        }
    }
}

impl AddAssign for PerftContext {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            nodes: self.nodes + other.nodes,
            captures: self.captures + other.captures,
            ep: self.ep + other.ep,
            castles: self.castles + other.castles,
            promotions: self.promotions + other.promotions,
            checks: self.checks + other.checks,
            disco_checks: self.disco_checks + other.disco_checks,
            double_checks: self.double_checks + other.double_checks,
            checkmates: self.checkmates + other.checkmates,
            elapsed: self.elapsed + other.elapsed,
        }
    }
}

impl fmt::Display for PerftContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Elapsed: {:.*}s, Nodes: {}, Captures: {}, EP: {}, Castles: {}, Promos: {}, Checks: {}, Discochecks: {}, Double checks: {}, Checkmates: {}", 
                3, (self.elapsed as f64) / 1_000_000_000_f64, self.nodes, self.captures, self.ep, self.castles, self.promotions,
                self.checks, self.disco_checks, self.double_checks, self.checkmates)
    }
}

pub trait Search {
    fn perft(&mut self, depth: u32) -> PerftContext;
    fn do_perft(&mut self, ctx: &mut PerftContext, depth: u32);
}

enum ThreadPoolMsg {
    NewJob(Job),
    Terminate,
}

enum WorkerMsg {
    PerftFinished { ctx: PerftContext },
}

trait FnBox {
    fn call_box(self: Box<Self>, tx: mpsc::Sender<WorkerMsg>);
}

impl<F: FnOnce(mpsc::Sender<WorkerMsg>)> FnBox for F {
    fn call_box(self: Box<F>, tx: mpsc::Sender<WorkerMsg>) {
        (*self)(tx);
    }
}

type Job = Box<dyn FnBox + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    tx: mpsc::Sender<ThreadPoolMsg>,
    rx: mpsc::Receiver<WorkerMsg>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        debug!("Creating new thread pool with {} workers", size);
        let mut workers = Vec::with_capacity(size);
        let (thr_pool_tx, thr_pool_rx) = channel();
        let (worker_tx, worker_rx) = channel();
        let thr_pool_rx = Arc::new(Mutex::new(thr_pool_rx));

        for id in 0..size {
            workers.push(Worker::new(
                id,
                mpsc::Sender::clone(&worker_tx),
                Arc::clone(&thr_pool_rx),
            ));
        }
        debug!("Created new thread pool");
        ThreadPool {
            workers,
            tx: thr_pool_tx,
            rx: worker_rx,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce(mpsc::Sender<WorkerMsg>) + Send + 'static,
    {
        let job = Box::new(f);
        debug!("Sending new job to workers");
        self.tx.send(ThreadPoolMsg::NewJob(job)).unwrap();
    }

    pub fn recv(&mut self) -> Result<WorkerMsg, mpsc::RecvError> {
        self.rx.recv()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        debug!("Sending terminate message to workers");

        for _ in &mut self.workers {
            self.tx.send(ThreadPoolMsg::Terminate).unwrap();
        }

        debug!("Shutting down workers");
        for worker in &mut self.workers {
            debug!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(
        id: usize,
        tx: mpsc::Sender<WorkerMsg>,
        rx: Arc<Mutex<mpsc::Receiver<ThreadPoolMsg>>>,
    ) -> Worker {
        let thread = thread::spawn(move || {
            debug!("Worker thread {} started", id);
            loop {
                debug!("Worker thread {} waiting", id);
                let msg = rx.lock().unwrap().recv().unwrap();

                match msg {
                    ThreadPoolMsg::NewJob(job) => {
                        debug!("Worker thread {} executing job", id);
                        job.call_box(mpsc::Sender::clone(&tx));
                    }
                    ThreadPoolMsg::Terminate => {
                        debug!("Worker thread {} terminating", id);
                        break;
                    }
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl Search for Board {
    fn perft(&mut self, depth: u32) -> PerftContext {
        let mut acc_ctx = PerftContext::new();

        let thread_count = num_cpus::get();
        let mut pool = ThreadPool::new(thread_count);

        let clock = Clock::new();
        let start = clock.now();

        let moves = self.generate_moves();
        let movecount = moves.len();

        for mov in moves {
            let mut board = self.clone();

            board.make_move(mov);
            pool.execute(move |tx| {
                let mut ctx = PerftContext::new();
                board.do_perft(&mut ctx, depth - 1);
                tx.send(WorkerMsg::PerftFinished { ctx });
            });
        }

        for i in 0..movecount {
            let res = pool.recv().unwrap();
            match res {
                WorkerMsg::PerftFinished { ctx } => acc_ctx += ctx,
            }
        }

        // loop {
        //     let msg = rx.recv().unwrap();
        //     match msg {
        //         WorkerMsg::PerftFinished{id, ctx} => {
        //             workers[id].join();
        //         }
        //     }
        // }

        // self.do_perft(&mut ctx, depth);

        let finish = clock.now();
        acc_ctx.elapsed = finish - start;
        acc_ctx
    }

    fn do_perft(&mut self, ctx: &mut PerftContext, depth: u32) {
        if depth == 0 {
            ctx.nodes += 1;
            // if !self.history().is_empty() {
            //     let mov = self.history().last().unwrap();
            //     if mov.is_capture() {
            //         ctx.captures += 1;
            //     }
            //     if mov.is_capture_en_passant() {
            //         ctx.ep += 1;
            //     }
            //     if mov.is_king_castle() || mov.is_queen_castle() {
            //         ctx.castles += 1;
            //     }
            //     if mov.is_promotion() {
            //         ctx.promotions += 1;
            //     }
            // }
            // let to_move = self.to_move();
            // if self.is_in_check(to_move) {
            //     ctx.checks += 1;
            //     // if self.is_mate(to_move) {
            //     //     ctx.checkmates += 1;
            //     // }
            // }
            return;
        }

        //let mut nodes = 0u64;
        let moves = self.generate_moves();
        for mov in moves.iter() {
            self.make_move(*mov);
            if !self.is_in_check(1 ^ self.to_move()) {
                self.do_perft(ctx, depth - 1);
            }
            self.unmake_move();
        }
    }
}
