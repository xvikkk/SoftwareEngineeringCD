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
- 0.1.0
    - 初始版本
- 0.1.1
    - 添加了竖轴移动和边界检查
- 0.1.2
    - 添加了无敌状态，玩家实体生成后的2秒内开启
### 操作
- 通过方向键控制玩家移动
- 按空格键发射子弹