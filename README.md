# rumo

[![Test](https://github.com/veoco/rumo/actions/workflows/test.yml/badge.svg)](https://github.com/veoco/rumo/actions/workflows/test.yml)

使用 Rust 编写的博客后端程序，与 Typecho 数据库级兼容（不含 mysql）。

测试网址：[点击跳转](https://rumo.cf)

## 目标

1. 前后端分离
2. 低内存占用
3. 数据库级兼容

## 基础架构

axum + sqlx + jwt + sqlite

## 路线图

**起步 - v0.9（已完成）：**

完成所有读取和写入 API、sqlite、mariadb 和 postgresql 支持，核心功能已可以替代原版。

**v0.9 - v1.0：**

优化代码结构，修复潜在错误和漏洞。

**v1.0 以后：**

待定

## 使用方法

配置通过以下环境变量获取：

- `DATABASE_URL`：必选，数据库 URL。
- `SECRET_KEY`：必选，密钥字符串，用于 jwt 加密。
- `LISTEN_ADDRESS`：可选，http 监听地址，默认为 127.0.0.1:3000。
- `TOKEN_EXPIRE`：可选，jwt 密钥过期时间，单位小时。
- `PRELOAD_INDEX`：可选，首页预加载，默认为 false。
- `INDEX_PAGE`：可选，预加载的首页文件地址，默认为当前目录下的 index.html 文件。
- `UPLOAD_ROOT`：可选，文件上传根目录，相当于原版 usr 文件夹所在目录，默认为当前工作目录。
- `READ_ONLY`：可选，只读模式将关闭所有写入 api，默认为 false。
- `TABLE_PREFIX`：可选，数据库表前缀，默认为 typecho_。

以下是 `systemd` 参考配置：

```ini
[Unit]
Description=rumo
After=network.target

[Service]
Environment="DATABASE_URL=sqlite:data.db"
Environment="SECRET_KEY=fake-key"
Environment="PRELOAD_INDEX=true"
Environment="INDEX_PAGE=/opt/rumo/index.html"
Environment="UPLOAD_ROOT=/opt/rumo"
Environment="RUST_LOG=ERROR"
WorkingDirectory=/opt/rumo
ExecStart=/opt/rumo/rumo run
User=rumo
Group=rumo

[Install]
WantedBy=multi-user.target
```

## 页面预加载说明

通过 [minijinja](https://crates.io/crates/minijinja) 支持类 jinja2/django 的写法，参考文件：

```html
<!DOCTYPE html>
<html lang="{{ options.lang }}">
<head>
  <meta charset="{{ options.charset }}" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{{ options.title }}</title>
  <script type="module" crossorigin src="/assets/fake.js"></script>
  <link rel="stylesheet" href="/assets/fake.css">
</head>
<body>
  <div id="app"></div>
</body>
</html>
```

可用变量请查看 typecho 数据库 options 表默认配置，或者利用选项相关 api 向 uid 为 0 的用户添加选项。

## Mysql/MariaDB 说明

rumo 兼容 mysql/mariadb，理论上由 rumo 生成的数据库是能够被 typecho 兼容的，实际未测试。

而 typehco 生成的数据库表使用了无符号整数，在目前架构下并不能与 rumo 兼容。

完全兼容需要相当大的工作量，目前没有兼容的计划，如有需要使用可尝试修改表中所有无符号整数列为有符号整数。

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
     - page：i32，>= 1
     - page_size：i32，>= 1
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
     - uid：i32

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
     - uid：i32

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
<summary>DELETE /api/users/:uid ，删除指定 uid 用户</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：禁止
    - PM0：允许

  2. 路径参数：
     - uid：i32

  3. 查询参数：
     - 无

  4. 提交表单：
     - 无
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

<details>
<summary>GET /api/users/:uid/options/ ，获取指定 uid 用户的选项列表</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：允许，仅当 uid 与登录用户相同
    - PM2：允许，仅当 uid 与登录用户相同
    - PM1：允许，仅当 uid 与登录用户相同
    - PM0：允许

  2. 路径参数：
     - uid: i32

  3. 查询参数：
     - 无
</details>

<details>
<summary>POST /api/users/:uid/options/，新建指定 uid 用户的选项</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：允许，仅当 uid 与登录用户相同
    - PM2：允许，仅当 uid 与登录用户相同
    - PM1：允许，仅当 uid 与登录用户相同
    - PM0：允许

  2. 路径参数：
     - uid: i32

  3. 查询参数：
     - 无
  
  4. 提交表单：
     - name：String，1 <= 长度 <= 32
     - value：String
</details>

<details>
<summary>GET /api/users/:uid/options/:name ，获取指定 uid 用户的指定 name 选项</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：允许，仅当 uid 与登录用户相同
    - PM2：允许，仅当 uid 与登录用户相同
    - PM1：允许，仅当 uid 与登录用户相同
    - PM0：允许

  2. 路径参数：
     - uid: i32
     - name: String

  3. 查询参数：
     - 无
</details>

<details>
<summary>PATCH /api/users/:uid/options/:name ，修改指定 uid 用户的指定 name 选项</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：允许，仅当 uid 与登录用户相同
    - PM2：允许，仅当 uid 与登录用户相同
    - PM1：允许，仅当 uid 与登录用户相同
    - PM0：允许

  2. 路径参数：
     - uid: i32
     - name: String

  3. 查询参数：
     - 无

  4. 提交表单：
     - value：String
</details>

<details>
<summary>DELETE /api/users/:uid/options/:name ，删除指定 uid 用户的指定 name 选项</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：允许，仅当 uid 与登录用户相同
    - PM2：允许，仅当 uid 与登录用户相同
    - PM1：允许，仅当 uid 与登录用户相同
    - PM0：允许

  2. 路径参数：
     - uid: i32
     - name: String

  3. 查询参数：
     - 无

  4. 提交表单：
     - 无
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
     - page：i32，>= 1
     - page_size：i32，>= 1
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
     - created：i32，unix 时间戳，精确到秒
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

<details>
<summary>PATCH /api/pages/:slug ，修改指定 slug 页面</summary>
  
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
     - title：String，1 <= 长度 <= 150
     - slug：String，1 <= 长度 <= 150
     - created：i32，unix 时间戳，精确到秒
     - text：String
     - template：Option<String>，1 <= 长度 <= 16
     - publish：Option<bool>，默认 true
     - allowComment：Option<bool>，默认 true
     - allowPing：Option<bool>，默认 true
     - allowFeed：Option<bool>，默认 true
</details>

<details>
<summary>DELETE /api/pages/:slug ，删除指定 slug 页面</summary>
  
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
</details>

<details>
<summary>POST /api/pages/:slug/fields/ ，新建指定 slug 页面的 field</summary>
  
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
     - name：String，1 <= 长度 <= 150
     - type：String，1 <= 长度 <= 8
     - str_value：Option<String>，仅当 type 为 str 时有效
     - int_value：Option<i32>，仅当 type 为 int 时有效
     - float_value：Option<f32>，仅当 type 为 float 时有效
</details>

<details>
<summary>GET /api/pages/:slug/fields/:name ，获取指定 slug 页面中指定 name 的 field</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String
     - name：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - 无
</details>

<details>
<summary>PATCH /api/pages/:slug/fields/:name ，修改指定 slug 页面中指定 name 的 field</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String
     - name：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - name：String，1 <= 长度 <= 150
     - type：String，1 <= 长度 <= 8
     - str_value：Option<String>，仅当 type 为 str 时有效
     - int_value：Option<i32>，仅当 type 为 int 时有效
     - float_value：Option<f32>，仅当 type 为 float 时有效
</details>

<details>
<summary>DELETE /api/pages/:slug/fields/:name ，删除改指定 slug 页面中指定 name 的 field</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String
     - name：String

  3. 查询参数：
     - 无

  4. 提交表单：
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
     - page：i32，>= 1
     - page_size：i32，>= 1
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
     - created：i32，unix 时间戳，精确到秒
     - text：String
     - status：String，1 <= 长度 <= 32
     - password：Option<String>，1 <= 长度 <= 32
     - allowComment：Option<bool>，默认 true
     - allowPing：Option<bool>，默认 true
     - allowFeed：Option<bool>，默认 true
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

<details>
<summary>PATCH /api/posts/:slug ，修改指定 slug 的文章</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - title：String，1 <= 长度 <= 150
     - slug：String，1 <= 长度 <= 150
     - created：i32，unix 时间戳，精确到秒
     - text：String
     - status：String，1 <= 长度 <= 32
     - password：Option<String>，1 <= 长度 <= 32
     - allowComment：Option<bool>，默认 true
     - allowPing：Option<bool>，默认 true
     - allowFeed：Option<bool>，默认 true
</details>

<details>
<summary>DELETE /api/posts/:slug ，删除指定 slug 的文章</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - 无
</details>

<details>
<summary>POST /api/posts/:slug/fields/ ，新建指定 slug 文章的 field</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许，仅当是用户是文章作者时允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - name：String，1 <= 长度 <= 150
     - type：String，1 <= 长度 <= 8
     - str_value：Option<String>，仅当 type 为 str 时有效
     - int_value：Option<i32>，仅当 type 为 int 时有效
     - float_value：Option<f32>，仅当 type 为 float 时有效
</details>

<details>
<summary>GET /api/posts/:slug/fields/:name ，获取指定 slug 文章中指定 name 的 field</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String
     - name：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - 无
</details>

<details>
<summary>PATCH /api/posts/:slug/fields/:name ，修改指定 slug 文章中指定 name 的 field</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许，仅当是用户是文章作者时允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String
     - name：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - name：String，1 <= 长度 <= 150
     - type：String，1 <= 长度 <= 8
     - str_value：Option<String>，仅当 type 为 str 时有效
     - int_value：Option<i32>，仅当 type 为 int 时有效
     - float_value：Option<f32>，仅当 type 为 float 时有效
</details>

<details>
<summary>DELETE /api/posts/:slug/fields/:name ，删除改指定 slug 文章中指定 name 的 field</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许，仅当是用户是文章作者时允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String
     - name：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - 无
</details>

### 分类相关 API：
<details>
<summary>GET /api/categories/ ，获取所有分类列表</summary>
  
 1. 权限要求：
    - PM4：允许
    - PM3：允许
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - 无

  3. 查询参数：
     - page：i32，>= 1
     - page_size：i32，>= 1
     - order_by：String，1 <= 长度 <= 13
</details>

<details>
<summary>POST /api/categories/ ，新建分类</summary>
  
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
     - parent：Option<i32>，> 0
</details>

<details>
<summary>GET /api/categories/:slug ，获取指定 slug 分类详情</summary>
  
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
<summary>PATCH /api/categories/:slug ，修改指定 slug 分类</summary>
  
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
     - name：String，1 <= 长度 <= 150
     - slug：String，1 <= 长度 <= 150
     - description：Option<String>，1 <= 长度 <= 150
     - parent：Option<i32>，> 0
</details>

<details>
<summary>DELETE /api/categories/:slug ，删除指定 slug 分类</summary>
  
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
     - 无
</details>

<details>
<summary>POST /api/categories/:slug/posts/ ，关联指定 slug 文章到指定 slug 分类</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - page：i32，>= 1
     - page_size：i32，>= 1
     - order_by：String，1 <= 长度 <= 13
     - private：bool，启用查询所有类型文章，仅 PM1 或更高权限可用
</details>

<details>
<summary>GET /api/categories/:slug/posts/ ，获取指定 slug 分类的所有文章列表</summary>
  
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
<summary>DELETE /api/categories/:slug/posts/:post_slug ，取消关联指定 slug 分类中的 post_slug 文章</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String
     - post_slug: String

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
     - page：i32，>= 1
     - page_size：i32，>= 1
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
     - parent：Option<i32>，> 0
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
<summary>PATCH /api/tags/:slug ，修改指定 slug 标签</summary>
  
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
     - name：String，1 <= 长度 <= 150
     - slug：String，1 <= 长度 <= 150
     - description：Option<String>，1 <= 长度 <= 150
     - parent：Option<i32>，> 0
</details>

<details>
<summary>DELETE /api/tags/:slug ，删除指定 slug 标签</summary>
  
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
     - page：i32，>= 1
     - page_size：i32，>= 1
     - order_by：String，1 <= 长度 <= 13
     - private：bool，启用查询所有类型文章，仅 PM1 或更高权限可用
</details>

<details>
<summary>DELETE /api/tags/:slug/posts/:post_slug ，取消关联指定 slug 标签的指定 post_slug 文章</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String
     - post_slug: String

  3. 查询参数：
     - 无
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
     - page：i32，>= 1
     - page_size：i32，>= 1
     - order_by：String，1 <= 长度 <= 13
</details>

<details>
<summary>GET /api/comments/:coid ，获取指定 coid 评论</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - coid：i32

  3. 查询参数：
     - 无
</details>

<details>
<summary>PATCH /api/comments/:coid ，修改指定 coid 评论</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - coid：i32

  3. 查询参数：
     - 无
   
  4. 提交表单：
     - text：String
     - status：String，1 <= 长度 <= 16
</details>

<details>
<summary>DELETE /api/comments/:coid ，删除指定 coid 评论</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - coid：i32

  3. 查询参数：
     - 无
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
     - page：i32，>= 1
     - page_size：i32，>= 1
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
     - parent：Option<i32>，> 0
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
     - page：i32，>= 1
     - page_size：i32，>= 1
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
     - parent：Option<i32>，> 0
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
     - page：i32，>= 1
     - page_size：i32，>= 1
     - order_by：String，1 <= 长度 <= 13
     - private：bool，启用查询所有用户附件，仅 PM1 或更高权限可用
</details>

<details>
<summary>GET /api/attachments/:cid ，获取指定 cid 附件列表</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许，仅当前用户上传附件
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - cid：i32

  3. 查询参数：
     - 无
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

<details>
<summary>PATCH /api/attachments/:cid ，修改指定 cid 附件</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许，仅当前用户上传附件
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - cid：i32

  3. 查询参数：
     - 无

  4. 提交表单：
     - file：multipart，multipart/form-data 单个文件，可用 `<input type="file" name="file">`
</details>

<details>
<summary>DELETE /api/attachments/:cid ，删除指定 cid 的附件</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - cid：i32

  3. 查询参数：
     - 无

  4. 提交表单：
     - 无
</details>

<details>
<summary>GET /api/pages/:slug/attachments/ ，获取指定 slug 页面所有附件列表</summary>
  
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
<summary>POST /api/pages/:slug/attachments/ ，关联指定 cid 附件到指定 slug 页面</summary>
  
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
     - cid：i32
</details>

<details>
<summary>DELETE /api/pages/:slug/attachments/:cid ，取消关联指定 slug 页面的指定 cid 的附件</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：禁止
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug: String
     - cid：i32

  3. 查询参数：
     - 无

  4. 提交表单：
     - 无
</details>

<details>
<summary>GET /api/posts/:slug/attachments/ ，获取指定 slug 文章所有附件列表</summary>
  
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
<summary>POST /api/posts/:slug/attachments/ ，关联指定 cid 附件到指定 slug 文章</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许，仅当前用户文章
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug：String

  3. 查询参数：
     - 无

  4. 提交表单：
     - cid：i32
</details>

<details>
<summary>DELETE /api/posts/:slug/attachments/:cid ，取消关联指定 slug 页面的指定 cid 的附件</summary>
  
 1. 权限要求：
    - PM4：禁止
    - PM3：禁止
    - PM2：允许，仅当前用户文章
    - PM1：允许
    - PM0：允许

  2. 路径参数：
     - slug: String
     - cid：i32

  3. 查询参数：
     - 无

  4. 提交表单：
     - 无
</details>
