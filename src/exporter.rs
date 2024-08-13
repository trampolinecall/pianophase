use std::path::PathBuf;

use macroquad::texture::get_screen_data;
use threadpool::ThreadPool;

pub struct Exporter {
    output_dir: PathBuf,
    current_frame: u32,
    thread_pool: ThreadPool,
    maximum_queue_size: usize,
}

impl Exporter {
    pub fn new(output_dir: PathBuf, num_export_threads: usize, maximum_queue_size: usize) -> Result<Exporter, Box<dyn std::error::Error>> {
        if !output_dir.exists() {
            std::fs::create_dir(&output_dir)?;
        }
        Ok(Exporter { output_dir, current_frame: 0, thread_pool: ThreadPool::new(num_export_threads), maximum_queue_size })
    }

    pub fn export_frame(&mut self) {
        let screen_image = get_screen_data();

        let mut output_path = self.output_dir.clone();
        output_path.push(format!("frame{:06}", self.current_frame));
        output_path.set_extension("png");

        self.thread_pool.execute({
            let current_frame = self.current_frame;
            move || {
                screen_image.export_png(output_path.to_str().unwrap());
                println!("frame {} exported", current_frame);
            }
        });
        self.current_frame += 1;

        if self.thread_pool.queued_count() > self.maximum_queue_size {
            println!(
                "maximum queue size reached; waiting for {} threads to finish until frame {}",
                self.thread_pool.queued_count(),
                self.current_frame
            );
            self.thread_pool.join();
        }
    }

    pub fn finish(&self) {
        println!("waiting for {} frames to finish exporting; total frame count {}", self.thread_pool.queued_count(), self.current_frame);
        self.thread_pool.join();
    }
}
