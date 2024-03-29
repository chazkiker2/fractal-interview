// We need a function that will evaluate the order that a set of tasks will be completed in.
// When idle, the CPU will take the next task that has been queued with the lowest time to complete.

// queued by moment in time
// keep CPU busy for exec duration
// seconds
// one task at a time
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Task {
    pub id: u64,
    pub queued_at: u32,
    pub execution_duration: u32,
}

fn remove_first<K: Clone + Ord, V>(map: &mut BTreeMap<K, V>) -> Option<V> {
    let key = map.keys().cloned().next();
    key.and_then(|k| map.remove(&k))
}

pub fn execution_order(mut tasks: Vec<Task>) -> Vec<u64> {
    let mut executed = vec![];

    tasks.sort_by_key(|task| task.queued_at);

    let mut time = 0_u32;
    let mut q: BTreeMap<(u32, u64), Task> = BTreeMap::new();

    // while there are still tasks to queue & execute
    while !tasks.is_empty() || !q.is_empty() {
        // look for any tasks with `queued_at` before/during the current time
        if !tasks.is_empty() {
            match tasks.iter().rposition(|task| task.queued_at <= time) {
                // add any tasks queued before/during the current time to the queue for execution
                Some(index) => q.extend(
                    tasks
                        .drain(..index + 1)
                        .into_iter()
                        .map(|task| ((task.execution_duration, task.id), task)),
                ),
                // otherwise, no tasks queued before this time range
                // so update time to match next task b/c computer is currently idle
                None => time = tasks.first().unwrap().queued_at,
            }
        }
        // execute any items in the queue
        if let Some(current_task) = remove_first(&mut q) {
            time += current_task.execution_duration;
            executed.push(current_task.id);
        }
    }

    executed
}

pub fn execution_order_original(mut tasks: Vec<Task>) -> Vec<u64> {
    if tasks.is_empty() {
        return vec![];
    }

    tasks.sort_by_key(|task| task.queued_at);

    let mut time = 0_u32;
    let mut result: Vec<u64> = vec![];

    loop {
        let current_task = tasks
            .iter()
            .take_while(|task| task.queued_at <= time)
            .min_by_key(|task| task.execution_duration);

        if let Some(current_task) = current_task {
            time += current_task.execution_duration;
            result.push(current_task.id);

            if tasks.is_empty() {
                break;
            }

            let index = tasks
                .iter()
                .position(|task| task.id == current_task.id)
                .unwrap();
            tasks.remove(index);
        } else if !tasks.is_empty() {
            time = tasks[0].queued_at;
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_queue_order() {
        // 44   0   2
        // 43   2   3
        // 42   5   3

        let tasks = vec![
            Task {
                id: 42,
                queued_at: 5,
                execution_duration: 3,
            },
            Task {
                id: 43,
                queued_at: 2,
                execution_duration: 3,
            },
            Task {
                id: 44,
                queued_at: 0,
                execution_duration: 2,
            },
        ];

        assert_eq!(execution_order(tasks), vec![44, 43, 42]);
    }

    #[test]
    fn two_items_queued_at_once() {
        // 0: #42 is queued
        // 0: #42 is started
        // 1: #43 is queued
        // 2: #44 is queued
        // 3: #42 is finished
        // 3: #44 is started (it is queued and has a lower execution_duration than #43)
        // 5: #44 is finished
        // 5: #43 is started
        // 8: #43 is finished

        let tasks = vec![
            Task {
                id: 42,
                queued_at: 0,
                execution_duration: 3,
            },
            Task {
                id: 43,
                queued_at: 1,
                execution_duration: 3,
            },
            Task {
                id: 44,
                queued_at: 2,
                execution_duration: 2,
            },
        ];

        assert_eq!(execution_order(tasks), vec![42, 44, 43]);
    }

    #[test]
    fn idle() {
        // 0: #42 is queued
        // 0: #42 is started
        // 1: #43 is queued
        // 2: #44 is queued
        // 3: #42 is finished
        // 3: #44 is started (it is queued and has a lower execution_duration than #43)
        // 5: #44 is finished
        // 5: #43 is started
        // 8: #43 is finished

        let tasks = vec![
            Task {
                id: 42,
                queued_at: 0,
                execution_duration: 1,
            },
            Task {
                id: 43,
                queued_at: 3,
                execution_duration: 3,
            },
        ];

        assert_eq!(execution_order(tasks), vec![42, 43]);
    }

    #[test]
    fn empty_task_list() {
        assert_eq!(execution_order(vec![]), vec![]);
    }

    #[test]
    fn two_items_same_queue_time_and_exec_duration() {
        let tasks = vec![
            Task {
                id: 42,
                queued_at: 1,
                execution_duration: 3,
            },
            Task {
                id: 43,
                queued_at: 1,
                execution_duration: 3,
            },
        ];

        assert_eq!(execution_order(tasks), vec![42, 43]);
    }

    #[test]
    fn task_inserted_into_queue() {
        // 0...
        //      queue 42
        //                  start 42
        // 1...
        //      queue 43
        // 2...
        //      queue 44
        // 3...
        //                              exec 42
        //                  start 43
        // 4...
        //
        // 5...
        //      queue 45
        // 6...
        //                              exec 43
        //                  start 44
        //                              exec 45
        // 7...
        //
        // 8...
        //                  exec 44
        let tasks = vec![
            Task {
                id: 42,
                queued_at: 0,
                execution_duration: 3,
            },
            Task {
                id: 43,
                queued_at: 1,
                execution_duration: 5,
            },
            Task {
                id: 44,
                queued_at: 2,
                execution_duration: 6,
            },
            Task {
                id: 45,
                queued_at: 5,
                execution_duration: 1,
            },
        ];

        assert_eq!(execution_order(tasks), vec![42, 43, 45, 44]);
    }
}

fn main() {
    println!("Hello, world!");
}
