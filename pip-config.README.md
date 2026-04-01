# PiP 小窗透明度配置

## 配置文件

`pip-config.json` - 小窗透明度配置文件

## 配置项

```json
{
  "pipOpacity": 0.3
}
```

- `pipOpacity`: 小窗透明度，范围 0.0 - 1.0
  - 0.0 = 完全透明
  - 0.3 = 30% 不透明（默认）
  - 1.0 = 完全不透明

## 使用方法

1. 编辑项目根目录的 `pip-config.json` 文件
2. 修改 `pipOpacity` 值（0.0 - 1.0）
3. 保存文件
4. 重新打开小窗即可生效

## 示例

**半透明（推荐）**
```json
{
  "pipOpacity": 0.5
}
```

**几乎透明**
```json
{
  "pipOpacity": 0.2
}
```

**完全不透明**
```json
{
  "pipOpacity": 1.0
}
```
