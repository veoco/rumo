# rumo

[![Test](https://github.com/veoco/rumo/actions/workflows/test.yml/badge.svg)](https://github.com/veoco/rumo/actions/workflows/test.yml)

使用 Rust 编写的博客后端程序，与 Typecho 数据库级兼容。

测试网址：[点击跳转](https://rumo.cf)

## 目标

1. 前后端分离
2. 低内存占用
3. 数据库级兼容

## 基础架构

axum + sqlx + jwt + sqlite

## 路线图

**起步 - v0.5（已完成）：**

仅支持 sqlite，完成主要读取 API，v0.5 版本可以用作前端主题开发。

**v0.5 - v1.0：**

完善读取和写入 API，添加 mariadb 和 postgresql 支持，预期 v1.0 版本可以完全替代原版。

**v1.0 以后：**

待定

## 使用方法

配置通过以下环境变量获取：

- `DATABASE_URL`：必选，数据库 URL，当前仅支持 sqlite
- `SECRET_KEY`：必选，密钥字符串，用于 jwt 加密
- `UPLOAD_ROOT`：可选，文件上传根目录，相当于原版 usr 文件夹所在目录，默认为当前工作目录。
- `READ_ONLY`：可选，只读模式将关闭所有写入 api，默认为 false。

以下是 `systemd` 参考配置：

```
[Unit]
Description=rumo
After=network.target

[Service]
Environment="DATABASE_URL=sqlite:data.db"
Environment="SECRET_KEY=fake-key"
Environment="UPLOAD_ROOT=/opt/rumo"
Environment="READ_ONLY=false"
Environment="RUST_LOG=ERROR"
WorkingDirectory=/opt/rumo
ExecStart=/opt/rumo/rumo run
User=rumo
Group=rumo

[Install]
WantedBy=multi-user.target
```

## API 列表

权限参考 typecho 的[文档](http://docs.typecho.org/develop/acl)：
 - PM0：PMAdministrator，对应 administrator（管理员）
 - PM1：PMEditor，对应 editor（编辑）
 - PM2：PMContributor，对应 contributor（贡献者）
 - PM3：PMSubscriber，对应 subscriber（关注者）
 - PM4：PMVisitor，对应 visitor（访问者）

查询参数无特别声明都是可选参数。

### 用户相关 API：
<details>
<summary>GET /api/users/ ，获取所有用户列表</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：禁止
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
</details>

<details>
<summary>GET /api/users/:uid ，获取指定 uid 用户信息</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：允许，仅当 uid 与登录用户相同
    - PM2：允许，仅当 uid 与登录用户相同
    - PM1：允许，仅当 uid 与登录用户相同
    - PM0：允许

  2. 路径参数：
     - uid：u32

  3. 查询参数：
     - 无
</details>

<details>
<summary>PACTH /api/users/:uid ，修改指定 uid 用户信息</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：允许，仅当 uid 与登录用户相同，禁止修改用户组
    - PM2：允许，仅当 uid 与登录用户相同，禁止修改用户组
    - PM1：允许，仅当 uid 与登录用户相同，禁止修改用户组
    - PM0：允许

  2. 路径参数：
     - uid：u32

  3. 查询参数：
     - 无

  4. 提交表单：
     - name：String，1 <= 长度 <= 32
     - screenName：String，1 <= 长度 <= 32
     - mail：String，邮箱格式
     - password：Option<String>，可选，1 <= 长度 <= 150，非空时仅更新 password
     - url：String，url 格式
     - group：String，6 <= 长度 <= 13
</details>

<details>
<summary>POST /api/users/token ，用户登录以获取 jwt 密钥</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - 无
  
  4. 提交表单：
     - mail：String，邮箱格式
     - password：String，长度 <= 150
</details>

<details>
<summary>POST /api/users ，用户注册</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - 无
  
  4. 提交表单：
     - name：String，1 <= 长度 <= 32
     - mail：String，邮箱格式
     - password：String，1 <= 长度 <= 150
     - url：String，url 格式
</details>

### 页面相关 API：
<details>
<summary>GET /api/pages/ ，获取所有页面列表</summary>

 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
     - private：Option<bool>，启用查询所有类型页面，默认 false，仅 PM1 或更高权限可用
</details>

<details>
<summary>POST /api/pages/ ，新建页面</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - 无

  4. 提交表单：
     - title：String，1 <= 长度 <= 150
     - slug：String，1 <= 长度 <= 150
     - created：u32，unix 时间戳，精确到秒
     - text：String
     - template：Option<String>，1 <= 长度 <= 16
     - publish：Option<bool>，默认 true
     - allowComment：Option<bool>，默认 true
     - allowPing：Option<bool>，默认 true
     - allowFeed：Option<bool>，默认 true
</details>

<details>
<summary>GET /api/pages/:slug ，获取指定 slug 页面详情，隐藏页面仅 PM1 或更高权限可获取</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无
</details>

### 文章相关 API：
<details>
<summary>GET /api/posts/ ，获取所有文章列表</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
     - private：bool，启用查询所有类型文章，仅 PM1 或更高权限可用
     - own: bool，启用查询当前用户所有文章，仅 PM3 或更高权限可用，与 private 同时使用时，两者均无效。
</details>

<details>
<summary>POST /api/posts/ ，新建文章</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - 无

  4. 提交表单：
     - title：String，1 <= 长度 <= 150
     - slug：String，1 <= 长度 <= 150
     - created：u32，unix 时间戳，精确到秒
     - text：String
     - template：Option<String>，1 <= 长度 <= 16
     - status：String，1 <= 长度 <= 32
     - password：Option<String>，1 <= 长度 <= 32
     - allowComment：String，长度 = 1
     - allowPing：String，长度 = 1
     - allowFeed：String，长度 = 1
</details>

<details>
<summary>GET /api/posts/:slug ，获取指定 slug 文章详情</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - password: String，1 <= 长度 <= 32
     - private：bool，启用查询所有类型文章，仅 PM1 或更高权限可用
</details>

### 分类相关 API：
<details>
<summary>GET /api/categories/ ，获取所有分类目录列表</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
</details>

<details>
<summary>POST /api/categories/ ，新建分类目录</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - 无

  4. 提交表单：
     - name：String，1 <= 长度 <= 150
     - slug：String，1 <= 长度 <= 150
     - description：Option<String>，1 <= 长度 <= 150
     - parent：Option<u32>，> 0
</details>

<details>
<summary>GET /api/categories/:slug ，获取指定 slug 分类目录详情</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无
</details>

<details>
<summary>POST /api/categories/:slug/posts/ ，关联指定 slug 文章到指定 slug 分类目录</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
     - private：bool，启用查询所有类型文章，仅 PM1 或更高权限可用
</details>

<details>
<summary>GET /api/categories/:slug/posts/ ，获取指定 slug 分类目录的所有文章列表</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无
</details>

### 标签相关 API：
<details>
<summary>GET /api/tags/ ，获取所有标签列表</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
</details>

<details>
<summary>POST /api/tags/ ，新建标签</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - 无

  4. 提交表单：
     - name：String，1 <= 长度 <= 150
     - slug：String，1 <= 长度 <= 150
     - description：Option<String>，1 <= 长度 <= 150
     - parent：Option<u32>，> 0
</details>

<details>
<summary>GET /api/tags/:slug ，获取指定 slug 标签详情</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无
</details>

<details>
<summary>POST /api/tags/:slug/posts/ ，关联指定 slug 文章到指定 slug 标签</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - slug：String，1 <= 长度 <= 150
</details>

<details>
<summary>GET /api/tags/:slug/posts/ ，获取指定 slug 标签的所有文章列表</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
     - private：bool，启用查询所有类型文章，仅 PM1 或更高权限可用
</details>

### 评论相关 API：
<details>
<summary>GET /api/comments/ ，获取所有评论列表</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
</details>

<details>
<summary>GET /api/pages/:slug/comments/ ，获取指定 slug 页面的评论列表</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
     - private：bool，启用查询所有类型页面的评论，仅 PM1 或更高权限可用
</details>

<details>
<summary>POST /api/pages/:slug/comments/ ，新建指定 slug 页面的评论</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - author：Option<String>，1 <= 长度 <= 150
     - mail：Option<String>，邮箱格式
     - url：Option<String>，url 格式
     - text: String
     - parent：Option<u32>，> 0
</details>

<details>
<summary>GET /api/posts/:slug/comments/ ，获取指定 slug 文章的评论列表</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
     - private：bool，启用查询所有类型文章的评论，仅 PM1 或更高权限可用
</details>

<details>
<summary>POST /api/posts/:slug/comments/ ，新建指定 slug 文章的评论</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - author：Option<String>，1 <= 长度 <= 150
     - mail：Option<String>，邮箱格式
     - url：Option<String>，url 格式
     - text: String
     - parent：Option<u32>，> 0
</details>

### 附件相关 API：
<details>
<summary>GET /api/attachments/ ，获取当前用户所有附件列表</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - page：u32，>= 1
     - page_size：u32，>= 1
     - order_by：String，1 <= 长度 <= 13
     - private：bool，启用查询所有用户附件，仅 PM1 或更高权限可用
</details>

<details>
<summary>POST /api/attachments/ ，新建附件</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - 无

  4. 提交表单：
     - file：multipart，multipart/form-data 单个文件，可用 `<input type="file" name="file">`
</details>
