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

**起步 - v0.5：**

仅支持 sqlite，完成用户、文章与页面、标签与分类、附件、评论五大模块的读取 API，以及部分必须的写入 API，预期 v0.5 版本 rumo 可以用作前端主题开发，但仍无法脱离原版 typecho。


**v0.5 - v1.0：**

完成五大模块写入 API，添加 mariadb 和 postgresql 支持，预期 v1.0 版本可以完全替代原版。

**v1.0 以后：**

待定

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
     - status：String，1 <= 长度 <= 32
     - password：Option<String>，1 <= 长度 <= 32
     - allowComment：String，长度 = 1
     - allowPing：String，长度 = 1
     - allowFeed：String，长度 = 1
</details>

<details>
<summary>GET /api/pages/:slug ，获取指定 slug 页面详情</summary>
  
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

### 分类目录相关 API：
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
<summary>GET /api/comments/:slug ，获取指定 slug 文章的评论列表</summary>
  
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
</details>

<details>
<summary>POST /api/comments/:slug ，新建指定 slug 文章的评论</summary>
  
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
