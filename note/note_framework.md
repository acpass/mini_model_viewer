# FrameWork

## Dependency

为了使用rust构建一个微型模型渲染器，采用了以下的依赖:
- winit: 用于窗口管理和事件处理
- ash: 用于提供Vulkan API的绑定
- glm: 用于数学计算，特别是矩阵和向量操作

## Design

### app.rs

依赖winit与graphics模块，负责应用程序的生命周期管理，包括窗口创建、事件循环和渲染调用。

### graphics.rs

提供图形后端渲染的trait，依赖model模块，实现事件循环和实际渲染逻辑的依赖反转，使得渲染器的实现与应用程序逻辑分离。

#### vulkan.rs

实现了GraphicsBackend trait，使用ash库进行Vulkan API的调用，负责Vulkan实例的创建、设备管理、交换链设置和渲染流程。

### model.rs

定义了模型数据结构，包括顶点、索引和纹理信息，提供了加载和管理模型资源的功能。
