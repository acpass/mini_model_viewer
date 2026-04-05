# Swap Chain创建

## 关键参数

- Device
- Surface

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
