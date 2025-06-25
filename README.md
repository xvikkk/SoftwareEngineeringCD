### 快速构建
- 通过VScode打开项目
- 打开终端
- 建议使用：
    - 'cargo run --features bevy/dynamic_linking'
- 或者：
    - 'cargo watch -q -c -x 'run --features bevy/dynamic_linking''
### 构建之前删除文件Cargo.lock
- rm Cargo.lock，这是一个版本锁

### 版本
- 1.0.0
    - 初始版本
- 1.0.1
    - 添加了竖轴移动和边界检查
- 1.0.2
    - 添加了无敌状态，玩家实体生成后的2秒内开启
