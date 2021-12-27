use regex::Regex;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug)]
pub struct Trajectory {
    pub frames: Vec<Frame>,
}

impl Trajectory {
    pub fn area(&self) -> (f32, f32, f32, f32) {
        let mut x_min = f32::MAX;
        let mut y_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_max = f32::MIN;

        for f in &self.frames {
            for p in &f.positions {
                x_min = f32::min(p[0], x_min);
                x_max = f32::max(p[0], x_max);
                y_min = f32::min(p[1], y_min);
                y_max = f32::max(p[1], y_max);
            }
        }
        (x_min, x_max, y_min, y_max)
    }
}

#[derive(Debug)]
pub struct Frame {
    pub positions: Vec<[f32; 2]>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
        }
    }
}

struct Entry {
    frame_id: i32,
    position: [f32; 2],
}

pub fn prase_trajectory_txt(path: &Path) -> Trajectory {
    let entry_matcher = Regex::new(r"^(\d+)\t(\d+)\t(\d+(?:\.\d+)?)\t(\d+(?:\.\d+)?)").unwrap();
    let file = std::fs::File::open(path).unwrap();
    let lines = BufReader::new(file).lines();
    let mut entries = Vec::<Entry>::new();
    for line in lines.flatten() {
        if let Some(captures) = entry_matcher.captures(line.as_ref()) {
            let frame_id = captures[2].parse::<i32>().unwrap();
            let x = captures[3].parse::<f32>().unwrap();
            let y = captures[4].parse::<f32>().unwrap();
            let position = [x, y];
            entries.push(Entry { frame_id, position })
        }
    }
    entries.sort_by(|a, b| a.frame_id.cmp(&b.frame_id));
    let mut trajectory = Trajectory { frames: Vec::new() };
    let mut last_index = -1;
    trajectory.frames.push(Frame::new());
    for entry in entries {
        if last_index < entry.frame_id {
            last_index += 1;
            trajectory.frames.push(Frame::new());
        }
        trajectory
            .frames
            .last_mut()
            .unwrap()
            .positions
            .push(entry.position);
    }
    trajectory
}

mod tests {
    use super::*;

    #[test]
    fn can_parse_trivial() {
        let path = std::path::Path::new("/Users/kkratz/Downloads/11_trains/results/train_traj.txt");
        let t = prase_trajectory_txt(path);
        println!("{:?}", t);
    }
}
