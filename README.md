### 快速构建
- 通过VScode打开项目
- 打开终端
- 建议使用：
    - “cargo run --features bevy/dynamic_linking”
- 或者：
    - “cargo watch -q -c -x 'run --features bevy/dynamic_linking'”
### 构建之前删除文件Cargo.lock
- rm Cargo.lock，这是一个版本锁