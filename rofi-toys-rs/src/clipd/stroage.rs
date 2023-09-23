use std::collections::VecDeque;

use crate::manager;

pub const UNLIMITED_SIZE: usize = 0;

#[derive(Debug)]
pub struct ClipboardStorage {
    max_size: usize,

    datas: VecDeque<manager::ClipboardData>,
}

impl ClipboardStorage {
    pub fn new(max_size: usize) -> ClipboardStorage {
        return ClipboardStorage {
            max_size,
            datas: VecDeque::new(),
        };
    }

    pub fn insert_data(&mut self, data: manager::ClipboardData) {
        if self.datas.len() == 0 {
            self.datas.push_front(data);
        } else if *self.datas.front().unwrap() != data {
            // 只有不相同的时候 push
            self.datas.push_front(data);
        }

        if self.max_size != UNLIMITED_SIZE {
            // 去掉多余的数据
            while self.datas.len() > self.max_size {
                self.datas.pop_back();
            }
        }
    }

    pub fn get_latest_data(&self) -> Option<&manager::ClipboardData> {
        return self.datas.front();
    }

    pub fn list(&self) -> Vec<&manager::ClipboardData> {
        self.datas.iter().collect::<Vec<_>>()
    }
    
    pub fn move_to_front(&mut self, idx: usize) -> Option<&manager::ClipboardData> {
        if let Some(data) = self.datas.remove(idx) {
            self.datas.push_front(data);
            return self.datas.front();
        } else {
            return None;
        }
    }
}
