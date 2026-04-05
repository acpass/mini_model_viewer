# Swap Chain创建

## 关键参数

- Device
- Surface

## swapchain功能

为渲染提供framebuffer，交换前台和后台图像以实现平滑显示。

## 创建过程

- 检查Device是否支持Swap Chain
- 添加Swap Chain拓展
- 查询Capabilities
- 获取Surface Format和Present Mode
- 通过Capabilities获取Swap Chain的Extent
- 创建khr::swapchain::Device函数加载器
- 调用函数加载器中的create_swapchain函数创建Swap Chain
- 调用函数加载器中的get_swapchain_images函数获取Swap Chain的图像列表
- 保存swap chain的format、present mode和extent以供后续使用

## 关键配置

- Surface Format: 选择合适的颜色格式和颜色空间
- Present Mode: 选择适合应用需求的呈现模式（如FIFO、MAILBOX、IMMEDIATE）
- Extent: 设置交换链图像的分辨率，通常与窗口大小一致
- Image Count: 设置交换链图像的数量，通常为2或3以实现双缓冲或三缓冲
