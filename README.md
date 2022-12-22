# rumo

[![Test](https://github.com/veoco/rumo/actions/workflows/test.yml/badge.svg)](https://github.com/veoco/rumo/actions/workflows/test.yml)

使用 Rust 编写的博客后端程序，与 Typecho 数据库级兼容。

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

- [x] GET  /api/users/ ，获取所有用户列表
- [x] GET  /api/users/:uid ，获取指定 uid 用户信息
- [x] PATH /api/users/:uid ，修改指定 uid 用户信息
- [x] POST /api/users/token ，用户登录以获取 jwt 密钥
- [x] POST /api/users ，用户注册
