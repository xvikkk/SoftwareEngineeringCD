### 快速构建
- cargo run --features bevy/dynamic_linking
- cargo watch -q -c -x 'run --features bevy/dynamic_linking'
### 构建之前删除文件Cargo.lock
- rm Cargo.lock，这是一个版本锁