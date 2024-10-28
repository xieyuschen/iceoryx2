#![feature(test)]

extern crate test;

#[cfg(test)]
mod sets_bench_test {
    use std::{
        collections::HashSet,
        sync::{Arc, Barrier, Mutex},
    };

    use iceoryx2_bb_lock_free::mpmc::bit_set::BitSet;
    use test::Bencher;

    struct LockSet {
        set: Arc<Mutex<HashSet<i64>>>,
    }
    impl LockSet {
        fn new() -> Self {
            return LockSet {
                set: Arc::new(Mutex::new(HashSet::new())),
            };
        }
        fn insert(&self, v: i64) -> bool {
            self.set.lock().unwrap().insert(v)
        }
    }
    static TURN: usize = 500;
    #[bench]
    fn bench_lock_set(b: &mut Bencher) {
        b.iter(|| {
            let set = &LockSet::new();
            let barrier = &Barrier::new(TURN + 1);

            std::thread::scope(|s| {
                // todo: write a note about why set_thread cannot be put outside
                let mut set_threads = vec![];
                for i in 0..TURN {
                    set_threads.push(s.spawn(move || {
                        // todo: we can not move if the set and barrier is not reference
                        barrier.wait();
                        set.insert(i as i64);
                    }));
                }
                barrier.wait();
                for t in set_threads {
                    t.join().unwrap();
                }
            });
        });
    }
    #[bench]
    fn bench_lockfree_set(b: &mut Bencher) {
        b.iter(|| {
            let set = &BitSet::new(TURN);
            let barrier = &Barrier::new(TURN + 1);

            std::thread::scope(|s| {
                let mut set_threads = vec![];
                for i in 0..TURN {
                    set_threads.push(s.spawn(move || {
                        barrier.wait();
                        set.set(i as usize);
                    }));
                }
                barrier.wait();
                for t in set_threads {
                    t.join().unwrap();
                }
            });
        });
    }
}

#[cfg(test)]
mod mpmc_container_bench_test {
    use std::sync::{Arc, Barrier, Mutex};

    use iceoryx2_bb_lock_free::mpmc::container::FixedSizeContainer;
    use test::Bencher;

    struct LockContainer {
        v: Arc<Mutex<Vec<i64>>>,
    }
    impl LockContainer {
        fn new() -> Self {
            return LockContainer {
                v: Arc::new(Mutex::new(vec![])),
            };
        }
        fn add(&self, v: i64) -> usize {
            let mut l = self.v.lock().unwrap();
            l.push(v);
            let len = l.len();
            drop(l);
            len
        }
    }
    static TURN: usize = 500;

    #[bench]
    fn bench_lock_container(b: &mut Bencher) {
        b.iter(|| {
            let set = &LockContainer::new();
            let barrier = &Barrier::new(TURN + 1);

            std::thread::scope(|s| {
                // todo: write a note about why set_thread cannot be put outside
                let mut set_threads = vec![];
                for i in 0..TURN {
                    set_threads.push(s.spawn(move || {
                        // todo: we can not move if the set and barrier is not reference
                        barrier.wait();
                        set.add(i as i64);
                    }));
                }
                barrier.wait();
                for t in set_threads {
                    t.join().unwrap();
                }
            });
        });
    }
    #[bench]
    fn bench_lockfree_container(b: &mut Bencher) {
        b.iter(|| {
            let sut = &FixedSizeContainer::<usize, TURN>::new();
            let barrier = &Barrier::new(TURN + 1);

            std::thread::scope(|s| {
                let mut set_threads = vec![];
                for i in 0..TURN {
                    set_threads.push(s.spawn(move || {
                        barrier.wait();
                        unsafe {
                            sut.add(i as usize).unwrap();
                        }
                    }));
                }
                barrier.wait();
                for t in set_threads {
                    t.join().unwrap();
                }
            });
        });
    }
}

#[cfg(test)]
mod spsc_queue_bench_test {
    use std::{sync::{Arc, Mutex}, thread};

    use iceoryx2_bb_lock_free::spsc::queue::Queue;
    use test::Bencher;

    struct LockIndexQueue {
        v: Arc<Mutex<Vec<i64>>>,
    }
    impl LockIndexQueue {
        fn new() -> Self {
            return LockIndexQueue {
                v: Arc::new(Mutex::new(vec![])),
            };
        }
        fn push(&self, v: i64) -> bool{
            self.v.lock().unwrap().push(v);
            true
        }
        fn pop(&self)-> Option<i64> {
            self.v.lock().unwrap().pop()
        }
    }
    
    #[bench]
    fn bench_lock_queue(b: &mut Bencher) {
        b.iter(|| {
            const LIMIT: i64 = 10000;
        
            let sut = LockIndexQueue::new();
            
            thread::scope(|s| {
                s.spawn(|| {
                    let mut counter: i64 = 0;
                    while counter <= LIMIT {
                        if sut.push(counter.clone()) {
                            counter += 1;
                        }
                    }
                });
        
                s.spawn(|| {
                    loop {
                        match sut.pop() {
                            Some(v) => {
                                if v == LIMIT {
                                    return;
                                }
                            }
                            None => (),
                        }
                    }
                });
            });
        });
    }
    #[bench]
    fn bench_lockfree_queue(b: &mut Bencher) {
        b.iter(|| {
            const LIMIT: i64 = 10000;
            const CAPACITY: usize = 1024;
        
            let sut = Queue::<i64, CAPACITY>::new();
            let mut sut_producer = sut.acquire_producer().unwrap();
            let mut sut_consumer = sut.acquire_consumer().unwrap();
            
            thread::scope(|s| {
                s.spawn(|| {
                    let mut counter: i64 = 0;
                    while counter <= LIMIT {
                        if sut_producer.push(&counter) {
                            counter += 1;
                        }
                    }
                });
        
                s.spawn(|| {
                    loop {
                        match sut_consumer.pop() {
                            Some(v) => {
                                if v == LIMIT {
                                    return;
                                }
                            }
                            None => (),
                        }
                    }
                });
            });
        });
    }
}
