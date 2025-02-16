use super::TimerSignal;
use crate::{utils::get_time_range, Result};

use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

/// A thread that sends a notification every 10th second
///
/// Timer will send an event to attached listeners (mpsc::Sender) every 10th
/// second (...10, ...20, ...)
///
/// The Timer thread will run continously until all Senders are dropped.
/// The Timer thread will be joined when all Senders are dropped.

#[derive(Debug, Default)]
pub struct Timer {
    /// A vector to store listeners (mpsc::Sender)
    txs: Arc<Mutex<Vec<Sender<TimerSignal>>>>,

    /// Thread handle
    pub handle: Option<JoinHandle<Result<()>>>,
}

impl Timer {
    /// Initialize Timer and run a thread to send events to attached listeners
    pub fn initialize(cycle: Duration) -> Result<Self> {
        let txs = Arc::new(Mutex::new(Vec::new()));

        // Add Default tx
        let (tx, _rx): (Sender<TimerSignal>, Receiver<TimerSignal>) = channel();
        txs.lock()?.push(tx);

        // Spawn a Thread
        let handle = Some({
            let txs = txs.clone();

            thread::spawn(move || {
                // Get remaining time for 10th second fire event
                let rem = get_time_range(0)?.rem;

                // Sleep for rem seconds
                thread::sleep(Duration::from_secs(rem));

                loop {
                    // Exit thread if there are no listeners
                    if txs.lock()?.len() == 0 {
                        return Ok(());
                    }

                    // Get current time
                    let current = TimerSignal::NextSnapshot(get_time_range(0)?.from);

                    // Iterate through Senders
                    txs.lock()?.iter().for_each(|tx| {
                        // Send event to attached Sender
                        let _res = tx.send(current);
                    });

                    // Sleep for 10s
                    thread::sleep(cycle);
                }
            })
        });

        Ok(Self { handle, txs })
    }

    /// Attach an mpsc::Sender to Timer
    ///
    /// Timer will dispatch an event with the timestamp of the current instant,
    /// every 10th second to all attached senders
    pub fn attach_listener(&mut self, tx: Sender<TimerSignal>) -> Result<()> {
        // Push Sender to a Vector of Sender(s)
        let txs = Arc::clone(&self.txs);
        txs.lock()?.push(tx);

        Ok(())
    }

    /// Clear the listeners (txs) from Timer. This will shutdown the Timer thread
    pub fn drop_listeners(&mut self) -> Result<()> {
        let txs = Arc::clone(&self.txs);
        txs.lock()?.clear();

        Ok(())
    }
}
