# 分发打包（Rust CLI）

本指南说明如何使用Rust CLI为其他用户创建发布数据归档。

## 命令

```bash
episteme dist
```

## `episteme dist`包含的内容
- `raw/`
- `meta/`
- `data/`（如果存在）
- `db/episteme.db`（嵌入数据库）

输出归档:
- `dist/episteme-data-<version>.tar.gz`

## 自动构建行为
- 如果`~/.episteme/db/episteme.db`不存在，`episteme dist`会先自动运行`epis build`。
- 构建后的数据库也会被复制到项目本地`db/`目录，以便包含在归档中。
- `epis install --local`从归档（或源码树回退）填充数据，并自动构建RAG索引到`~/.episteme/`。

## 选项
- `--out-dir <DIR>`: 输出目录（默认: `dist`）
- `--no-db`: 跳过数据库包含
- `--skip-build`: 数据库缺失时不自动构建

示例:

```bash
# 默认打包到dist/
episteme dist

# 自定义输出目录
episteme dist --out-dir release

# 仅打包元数据（不含数据库）
episteme dist --no-db

# 严格模式: 数据库缺失时报错
episteme dist --skip-build
```

## 验证
生成归档后，验证结构:

```bash
tar -tzf dist/episteme-data-*.tar.gz | head -n 30
```

你应该能看到以下条目:
- `episteme-data-<version>/raw/...`
- `episteme-data-<version>/meta/...`
- `episteme-data-<version>/db/episteme.db`（除非使用了`--no-db`）
