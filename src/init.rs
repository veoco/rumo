use sqlx::any::AnyKind;
use sqlx::Executor;
use std::time::SystemTime;

use super::users::{models::UserRegister, utils::hash};
use super::AppState;

pub async fn init_table(state: &AppState) {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => {
            r#"
            CREATE SEQUENCE "typecho_comments_seq";
            CREATE TABLE "typecho_comments" (
                "coid" INT NOT NULL DEFAULT nextval('typecho_comments_seq'),
                "cid" INT NULL DEFAULT '0',
                "created" INT NULL DEFAULT '0',
                "author" VARCHAR(150) NULL DEFAULT NULL,
                "authorId" INT NULL DEFAULT '0',
                "ownerId" INT NULL DEFAULT '0',
                "mail" VARCHAR(150) NULL DEFAULT NULL,
                "url" VARCHAR(255) NULL DEFAULT NULL,
                "ip" VARCHAR(64) NULL DEFAULT NULL,
                "agent" VARCHAR(511) NULL DEFAULT NULL,
                "text" TEXT NULL DEFAULT NULL,
                "type" VARCHAR(16) NULL DEFAULT 'comment',
                "status" VARCHAR(16) NULL DEFAULT 'approved',
                "parent" INT NULL DEFAULT '0',
                PRIMARY KEY ("coid")
            );
            CREATE INDEX "typecho_comments_cid" ON "typecho_comments" ("cid");
            CREATE INDEX "typecho_comments_created" ON "typecho_comments" ("created");

            CREATE SEQUENCE "typecho_contents_seq";
            CREATE TABLE "typecho_contents" (
                "cid" INT NOT NULL DEFAULT nextval('typecho_contents_seq'),
                "title" VARCHAR(150) NULL DEFAULT NULL,
                "slug" VARCHAR(150) NULL DEFAULT NULL,
                "created" INT NULL DEFAULT '0',
                "modified" INT NULL DEFAULT '0',
                "text" TEXT NULL DEFAULT NULL,
                "order" INT NULL DEFAULT '0',
                "authorId" INT NULL DEFAULT '0',
                "template" VARCHAR(32) NULL DEFAULT NULL,
                "type" VARCHAR(16) NULL DEFAULT 'post',
                "status" VARCHAR(16) NULL DEFAULT 'publish',
                "password" VARCHAR(32) NULL DEFAULT NULL,
                "commentsNum" INT NULL DEFAULT '0',
                "allowComment" CHAR(1) NULL DEFAULT '0',
                "allowPing" CHAR(1) NULL DEFAULT '0',
                "allowFeed" CHAR(1) NULL DEFAULT '0',
                "parent" INT NULL DEFAULT '0',
                PRIMARY KEY ("cid"),
                UNIQUE ("slug")
            );
            CREATE INDEX "typecho_contents_created" ON "typecho_contents" ("created");
            
            CREATE TABLE "typecho_fields" (
                "cid" INT NOT NULL,
                "name" VARCHAR(150) NOT NULL,
                "type" VARCHAR(8) NULL DEFAULT 'str',
                "str_value" TEXT NULL DEFAULT NULL,
                "int_value" INT NULL DEFAULT '0',
                "float_value" REAL NULL DEFAULT '0',
                PRIMARY KEY  ("cid","name")
            );
            CREATE INDEX "typecho_fields_int_value" ON "typecho_fields" ("int_value");
            CREATE INDEX "typecho_fields_float_value" ON "typecho_fields" ("float_value");
            
            CREATE SEQUENCE "typecho_metas_seq";
            CREATE TABLE "typecho_metas" (
                "mid" INT NOT NULL DEFAULT nextval('typecho_metas_seq'),
                "name" VARCHAR(150) NULL DEFAULT NULL,
                "slug" VARCHAR(150) NULL DEFAULT NULL,
                "type" VARCHAR(16) NOT NULL DEFAULT '',
                "description" VARCHAR(150) NULL DEFAULT NULL,
                "count" INT NULL DEFAULT '0',
                "order" INT NULL DEFAULT '0',
                "parent" INT NULL DEFAULT '0',
                PRIMARY KEY ("mid")
            );
            CREATE INDEX "typecho_metas_slug" ON "typecho_metas" ("slug");
            
            CREATE TABLE "typecho_options" (
                "name" VARCHAR(32) NOT NULL DEFAULT '',
                "user" INT NOT NULL DEFAULT '0',
                "value" TEXT NULL DEFAULT NULL,
                PRIMARY KEY ("name","user")
            );
            
            CREATE TABLE "typecho_relationships" (
                "cid" INT NOT NULL DEFAULT '0',
                "mid" INT NOT NULL DEFAULT '0',
                PRIMARY KEY ("cid","mid")
            ); 
            
            CREATE SEQUENCE "typecho_users_seq";
            CREATE TABLE "typecho_users" (
                "uid" INT NOT NULL DEFAULT nextval('typecho_users_seq') ,
                "name" VARCHAR(32) NULL DEFAULT NULL,
                "password" VARCHAR(64) NULL DEFAULT NULL,
                "mail" VARCHAR(150) NULL DEFAULT NULL,
                "url" VARCHAR(150) NULL DEFAULT NULL,
                "screenName" VARCHAR(32) NULL DEFAULT NULL,
                "created" INT NULL DEFAULT '0',
                "activated" INT NULL DEFAULT '0',
                "logged" INT NULL DEFAULT '0',
                "group" VARCHAR(16) NULL DEFAULT 'visitor',
                "authCode" VARCHAR(64) NULL DEFAULT NULL,
                PRIMARY KEY ("uid"),
                UNIQUE ("name"),
                UNIQUE ("mail")
            );
            "#
        }
        AnyKind::MySql => {
            r#"
            CREATE TABLE `typecho_comments` (
                `coid` int(10) NOT NULL auto_increment,
                `cid` int(10) default '0',
                `created` int(10) default '0',
                `author` varchar(150) default NULL,
                `authorId` int(10) default '0',
                `ownerId` int(10) default '0',
                `mail` varchar(150) default NULL,
                `url` varchar(255) default NULL,
                `ip` varchar(64) default NULL,
                `agent` varchar(511) default NULL,
                `text` text,
                `type` varchar(16) default 'comment',
                `status` varchar(16) default 'approved',
                `parent` int(10) default '0',
                PRIMARY KEY  (`coid`),
                KEY `cid` (`cid`),
                KEY `created` (`created`)
            ) ENGINE=InnoDB  DEFAULT CHARSET=utf8mb4;
          
            CREATE TABLE `typecho_contents` (
                `cid` int(10) NOT NULL auto_increment,
                `title` varchar(150) default NULL,
                `slug` varchar(150) default NULL,
                `created` int(10) default '0',
                `modified` int(10) default '0',
                `text` longtext,
                `order` int(10) default '0',
                `authorId` int(10) default '0',
                `template` varchar(32) default NULL,
                `type` varchar(16) default 'post',
                `status` varchar(16) default 'publish',
                `password` varchar(32) default NULL,
                `commentsNum` int(10) default '0',
                `allowComment` char(1) default '0',
                `allowPing` char(1) default '0',
                `allowFeed` char(1) default '0',
                `parent` int(10) default '0',
                PRIMARY KEY  (`cid`),
                UNIQUE KEY `slug` (`slug`),
                KEY `created` (`created`)
            ) ENGINE=InnoDB  DEFAULT CHARSET=utf8mb4;
          
            CREATE TABLE `typecho_fields` (
                `cid` int(10) NOT NULL,
                `name` varchar(150) NOT NULL,
                `type` varchar(8) default 'str',
                `str_value` text,
                `int_value` int(10) default '0',
                `float_value` float default '0',
                PRIMARY KEY  (`cid`,`name`),
                KEY `int_value` (`int_value`),
                KEY `float_value` (`float_value`)
            ) ENGINE=InnoDB  DEFAULT CHARSET=utf8mb4;
          
            CREATE TABLE `typecho_metas` (
                `mid` int(10) NOT NULL auto_increment,
                `name` varchar(150) default NULL,
                `slug` varchar(150) default NULL,
                `type` varchar(32) NOT NULL,
                `description` varchar(150) default NULL,
                `count` int(10) default '0',
                `order` int(10) default '0',
                `parent` int(10) default '0',
                PRIMARY KEY  (`mid`),
                KEY `slug` (`slug`)
            ) ENGINE=InnoDB  DEFAULT CHARSET=utf8mb4;
          
            CREATE TABLE `typecho_options` (
                `name` varchar(32) NOT NULL,
                `user` int(10) NOT NULL default '0',
                `value` text,
                PRIMARY KEY  (`name`,`user`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
          
            CREATE TABLE `typecho_relationships` (
                `cid` int(10) NOT NULL,
                `mid` int(10) NOT NULL,
                PRIMARY KEY  (`cid`,`mid`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
          
            CREATE TABLE `typecho_users` (
                `uid` int(10) NOT NULL auto_increment,
                `name` varchar(32) default NULL,
                `password` varchar(64) default NULL,
                `mail` varchar(150) default NULL,
                `url` varchar(150) default NULL,
                `screenName` varchar(32) default NULL,
                `created` int(10) default '0',
                `activated` int(10) default '0',
                `logged` int(10) default '0',
                `group` varchar(16) default 'visitor',
                `authCode` varchar(64) default NULL,
                PRIMARY KEY  (`uid`),
                UNIQUE KEY `name` (`name`),
                UNIQUE KEY `mail` (`mail`)
            ) ENGINE=InnoDB  DEFAULT CHARSET=utf8mb4;
            "#
        }
        AnyKind::Sqlite => {
            r#"
            CREATE TABLE typecho_comments (
                "coid" INTEGER NOT NULL PRIMARY KEY,
                "cid" int(10) default '0' ,
                "created" int(10) default '0' ,
                "author" varchar(150) default NULL ,
                "authorId" int(10) default '0' ,
                "ownerId" int(10) default '0' ,
                "mail" varchar(150) default NULL ,
                "url" varchar(255) default NULL ,
                "ip" varchar(64) default NULL , 
                "agent" varchar(511) default NULL ,
                "text" text , 
                "type" varchar(16) default 'comment' , 
                "status" varchar(16) default 'approved' , 
                "parent" int(10) default '0'
            );
            CREATE INDEX typecho_comments_cid ON typecho_comments ("cid");
            CREATE INDEX typecho_comments_created ON typecho_comments ("created");
        
            CREATE TABLE typecho_contents (
                "cid" INTEGER NOT NULL PRIMARY KEY, 
                "title" varchar(150) default NULL ,
                "slug" varchar(150) default NULL ,
                "created" int(10) default '0' , 
                "modified" int(10) default '0' , 
                "text" text , 
                "order" int(10) default '0' , 
                "authorId" int(10) default '0' , 
                "template" varchar(32) default NULL , 
                "type" varchar(16) default 'post' , 
                "status" varchar(16) default 'publish' , 
                "password" varchar(32) default NULL , 
                "commentsNum" int(10) default '0' , 
                "allowComment" char(1) default '0' , 
                "allowPing" char(1) default '0' , 
                "allowFeed" char(1) default '0' ,
                "parent" int(10) default '0'
            );
            CREATE UNIQUE INDEX typecho_contents_slug ON typecho_contents ("slug");
            CREATE INDEX typecho_contents_created ON typecho_contents ("created");
        
            CREATE TABLE "typecho_fields" (
                "cid" INTEGER NOT NULL,
                "name" varchar(150) NOT NULL,
                "type" varchar(8) default 'str',
                "str_value" text,
                "int_value" int(10) default '0',
                "float_value" real default '0'
            );
            CREATE UNIQUE INDEX typecho_fields_cid_name ON typecho_fields ("cid", "name");
            CREATE INDEX typecho_fields_int_value ON typecho_fields ("int_value");
            CREATE INDEX typecho_fields_float_value ON typecho_fields ("float_value");
        
            CREATE TABLE typecho_metas (
                "mid" INTEGER NOT NULL PRIMARY KEY, 
                "name" varchar(150) default NULL ,
                "slug" varchar(150) default NULL ,
                "type" varchar(32) NOT NULL , 
                "description" varchar(150) default NULL ,
                "count" int(10) default '0' , 
                "order" int(10) default '0' ,
                "parent" int(10) default '0'
            );
            CREATE INDEX typecho_metas_slug ON typecho_metas ("slug");
        
            CREATE TABLE typecho_options (
                "name" varchar(32) NOT NULL , 
                "user" int(10) NOT NULL default '0' , 
                "value" text
            );
            CREATE UNIQUE INDEX typecho_options_name_user ON typecho_options ("name", "user");
        
            CREATE TABLE typecho_relationships (
                "cid" int(10) NOT NULL , 
                "mid" int(10) NOT NULL
            );
            CREATE UNIQUE INDEX typecho_relationships_cid_mid ON typecho_relationships ("cid", "mid");
        
            CREATE TABLE typecho_users (
                "uid" INTEGER NOT NULL PRIMARY KEY, 
                "name" varchar(32) default NULL ,
                "password" varchar(64) default NULL , 
                "mail" varchar(150) default NULL ,
                "url" varchar(150) default NULL ,
                "screenName" varchar(32) default NULL , 
                "created" int(10) default '0' , 
                "activated" int(10) default '0' , 
                "logged" int(10) default '0' , 
                "group" varchar(16) default 'visitor' , 
                "authCode" varchar(64) default NULL
            );
            CREATE UNIQUE INDEX typecho_users_name ON typecho_users ("name");
            CREATE UNIQUE INDEX typecho_users_mail ON typecho_users ("mail");
            "#
        }
    };
    let mut conn = state.pool.acquire().await.expect("open database failed");
    let _ = conn.execute(sql).await.expect("database already exists");
}

pub async fn init_admin(state: &AppState, user_register: UserRegister) {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32;
    let hashed_password = hash(&user_register.password);

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            INSERT INTO {users_table} ("name", "mail", "url", "screenName", "password", "created", "group")
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            users_table = &state.users_table,
        ),
        AnyKind::MySql => format!(
            r#"
            INSERT INTO {users_table} (`name`, `mail`, `url`, `screenName`, `password`, `created`, `group`)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            users_table = &state.users_table,
        ),
        _ => format!(
            r#"
            INSERT INTO {users_table} ("name", "mail", "url", "screenName", "password", "created", "group")
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            users_table = &state.users_table,
        ),
    };
    sqlx::query(&sql)
        .bind(&user_register.name)
        .bind(user_register.mail)
        .bind(user_register.url)
        .bind(&user_register.name)
        .bind(hashed_password)
        .bind(now)
        .bind("administrator")
        .execute(&state.pool)
        .await
        .expect("user already exists");
}

pub async fn init_options(state: &AppState) {
    let secret = format!("{}", &state.secret_key);
    let options = [
        ["theme", "default"],
        [
            "theme:default",
            r#"a:2:{s:7:""logoUrl"";N;s:12:""sidebarBlock"";a:5:{i:0;s:15:""ShowRecentPosts"";i:1;s:18:""ShowRecentComments"";i:2;s:12:""ShowCategory"";i:3;s:11:""ShowArchive"";i:4;s:9:""ShowOther"";}}"#,
        ],
        ["timezone", "28800"],
        ["lang", "zh_CN"],
        ["charset", "UTF-8"],
        ["contentType", "text/html"],
        ["gzip", "0"],
        ["generator", "Typecho 1.2.0"],
        ["title", "Hello World"],
        ["description", "Your description here."],
        ["keywords", "typecho,php,blog"],
        ["rewrite", "1"],
        ["frontPage", "recent"],
        ["frontArchive", "0"],
        ["commentsRequireMail", "1"],
        ["commentsWhitelist", "0"],
        ["commentsRequireURL", "0"],
        ["commentsRequireModeration", "0"],
        ["plugins", r#"a:0:{}"#],
        ["commentDateFormat", r#"F jS, Y \a\t h:i a"#],
        ["siteUrl", "https://rumo.cf"],
        ["defaultCategory", "1"],
        ["allowRegister", "0"],
        ["defaultAllowComment", "1"],
        ["defaultAllowPing", "1"],
        ["defaultAllowFeed", "1"],
        ["pageSize", "5"],
        ["postsListSize", "10"],
        ["commentsListSize", "10"],
        ["commentsHTMLTagAllowed", ""],
        ["postDateFormat", "Y-m-d"],
        ["feedFullText", "1"],
        ["editorSize", "350"],
        ["autoSave", "0"],
        ["markdown", "1"],
        ["xmlrpcMarkdown", "0"],
        ["commentsMaxNestingLevels", "5"],
        ["commentsPostTimeout", "2592000"],
        ["commentsUrlNofollow", "1"],
        ["commentsShowUrl", "1"],
        ["commentsMarkdown", "0"],
        ["commentsPageBreak", "0"],
        ["commentsThreaded", "1"],
        ["commentsPageSize", "20"],
        ["commentsPageDisplay", "last"],
        ["commentsOrder", "ASC"],
        ["commentsCheckReferer", "1"],
        ["commentsAutoClose", "0"],
        ["commentsPostIntervalEnable", "1"],
        ["commentsPostInterval", "60"],
        ["commentsShowCommentOnly", "0"],
        ["commentsAvatar", "1"],
        ["commentsAvatarRating", "G"],
        ["commentsAntiSpam", "1"],
        [
            "routingTable",
            r#"a:26:{i:0;a:25:{s:5:""index"";a:6:{s:3:""url"";s:1:""/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:8:""|^[/]?$|"";s:6:""format"";s:1:""/"";s:6:""params"";a:0:{}}s:7:""archive"";a:6:{s:3:""url"";s:6:""/blog/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:13:""|^/blog[/]?$|"";s:6:""format"";s:6:""/blog/"";s:6:""params"";a:0:{}}s:2:""do"";a:6:{s:3:""url"";s:22:""/action/[action:alpha]"";s:6:""widget"";s:14:""\Widget\Action"";s:6:""action"";s:6:""action"";s:4:""regx"";s:32:""|^/action/([_0-9a-zA-Z-]+)[/]?$|"";s:6:""format"";s:10:""/action/%s"";s:6:""params"";a:1:{i:0;s:6:""action"";}}s:4:""post"";a:6:{s:3:""url"";s:24:""/archives/[cid:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:26:""|^/archives/([0-9]+)[/]?$|"";s:6:""format"";s:13:""/archives/%s/"";s:6:""params"";a:1:{i:0;s:3:""cid"";}}s:10:""attachment"";a:6:{s:3:""url"";s:26:""/attachment/[cid:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:28:""|^/attachment/([0-9]+)[/]?$|"";s:6:""format"";s:15:""/attachment/%s/"";s:6:""params"";a:1:{i:0;s:3:""cid"";}}s:8:""category"";a:6:{s:3:""url"";s:17:""/category/[slug]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:25:""|^/category/([^/]+)[/]?$|"";s:6:""format"";s:13:""/category/%s/"";s:6:""params"";a:1:{i:0;s:4:""slug"";}}s:3:""tag"";a:6:{s:3:""url"";s:12:""/tag/[slug]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:20:""|^/tag/([^/]+)[/]?$|"";s:6:""format"";s:8:""/tag/%s/"";s:6:""params"";a:1:{i:0;s:4:""slug"";}}s:6:""author"";a:6:{s:3:""url"";s:22:""/author/[uid:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:24:""|^/author/([0-9]+)[/]?$|"";s:6:""format"";s:11:""/author/%s/"";s:6:""params"";a:1:{i:0;s:3:""uid"";}}s:6:""search"";a:6:{s:3:""url"";s:19:""/search/[keywords]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:23:""|^/search/([^/]+)[/]?$|"";s:6:""format"";s:11:""/search/%s/"";s:6:""params"";a:1:{i:0;s:8:""keywords"";}}s:10:""index_page"";a:6:{s:3:""url"";s:21:""/page/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:22:""|^/page/([0-9]+)[/]?$|"";s:6:""format"";s:9:""/page/%s/"";s:6:""params"";a:1:{i:0;s:4:""page"";}}s:12:""archive_page"";a:6:{s:3:""url"";s:26:""/blog/page/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:27:""|^/blog/page/([0-9]+)[/]?$|"";s:6:""format"";s:14:""/blog/page/%s/"";s:6:""params"";a:1:{i:0;s:4:""page"";}}s:13:""category_page"";a:6:{s:3:""url"";s:32:""/category/[slug]/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:34:""|^/category/([^/]+)/([0-9]+)[/]?$|"";s:6:""format"";s:16:""/category/%s/%s/"";s:6:""params"";a:2:{i:0;s:4:""slug"";i:1;s:4:""page"";}}s:8:""tag_page"";a:6:{s:3:""url"";s:27:""/tag/[slug]/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:29:""|^/tag/([^/]+)/([0-9]+)[/]?$|"";s:6:""format"";s:11:""/tag/%s/%s/"";s:6:""params"";a:2:{i:0;s:4:""slug"";i:1;s:4:""page"";}}s:11:""author_page"";a:6:{s:3:""url"";s:37:""/author/[uid:digital]/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:33:""|^/author/([0-9]+)/([0-9]+)[/]?$|"";s:6:""format"";s:14:""/author/%s/%s/"";s:6:""params"";a:2:{i:0;s:3:""uid"";i:1;s:4:""page"";}}s:11:""search_page"";a:6:{s:3:""url"";s:34:""/search/[keywords]/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:32:""|^/search/([^/]+)/([0-9]+)[/]?$|"";s:6:""format"";s:14:""/search/%s/%s/"";s:6:""params"";a:2:{i:0;s:8:""keywords"";i:1;s:4:""page"";}}s:12:""archive_year"";a:6:{s:3:""url"";s:18:""/[year:digital:4]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:19:""|^/([0-9]{4})[/]?$|"";s:6:""format"";s:4:""/%s/"";s:6:""params"";a:1:{i:0;s:4:""year"";}}s:13:""archive_month"";a:6:{s:3:""url"";s:36:""/[year:digital:4]/[month:digital:2]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:30:""|^/([0-9]{4})/([0-9]{2})[/]?$|"";s:6:""format"";s:7:""/%s/%s/"";s:6:""params"";a:2:{i:0;s:4:""year"";i:1;s:5:""month"";}}s:11:""archive_day"";a:6:{s:3:""url"";s:52:""/[year:digital:4]/[month:digital:2]/[day:digital:2]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:41:""|^/([0-9]{4})/([0-9]{2})/([0-9]{2})[/]?$|"";s:6:""format"";s:10:""/%s/%s/%s/"";s:6:""params"";a:3:{i:0;s:4:""year"";i:1;s:5:""month"";i:2;s:3:""day"";}}s:17:""archive_year_page"";a:6:{s:3:""url"";s:38:""/[year:digital:4]/page/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:33:""|^/([0-9]{4})/page/([0-9]+)[/]?$|"";s:6:""format"";s:12:""/%s/page/%s/"";s:6:""params"";a:2:{i:0;s:4:""year"";i:1;s:4:""page"";}}s:18:""archive_month_page"";a:6:{s:3:""url"";s:56:""/[year:digital:4]/[month:digital:2]/page/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:44:""|^/([0-9]{4})/([0-9]{2})/page/([0-9]+)[/]?$|"";s:6:""format"";s:15:""/%s/%s/page/%s/"";s:6:""params"";a:3:{i:0;s:4:""year"";i:1;s:5:""month"";i:2;s:4:""page"";}}s:16:""archive_day_page"";a:6:{s:3:""url"";s:72:""/[year:digital:4]/[month:digital:2]/[day:digital:2]/page/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:55:""|^/([0-9]{4})/([0-9]{2})/([0-9]{2})/page/([0-9]+)[/]?$|"";s:6:""format"";s:18:""/%s/%s/%s/page/%s/"";s:6:""params"";a:4:{i:0;s:4:""year"";i:1;s:5:""month"";i:2;s:3:""day"";i:3;s:4:""page"";}}s:12:""comment_page"";a:6:{s:3:""url"";s:53:""[permalink:string]/comment-page-[commentPage:digital]"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:36:""|^(.+)/comment\-page\-([0-9]+)[/]?$|"";s:6:""format"";s:18:""%s/comment-page-%s"";s:6:""params"";a:2:{i:0;s:9:""permalink"";i:1;s:11:""commentPage"";}}s:4:""feed"";a:6:{s:3:""url"";s:20:""/feed[feed:string:0]"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:4:""feed"";s:4:""regx"";s:17:""|^/feed(.*)[/]?$|"";s:6:""format"";s:7:""/feed%s"";s:6:""params"";a:1:{i:0;s:4:""feed"";}}s:8:""feedback"";a:6:{s:3:""url"";s:31:""[permalink:string]/[type:alpha]"";s:6:""widget"";s:16:""\Widget\Feedback"";s:6:""action"";s:6:""action"";s:4:""regx"";s:29:""|^(.+)/([_0-9a-zA-Z-]+)[/]?$|"";s:6:""format"";s:5:""%s/%s"";s:6:""params"";a:2:{i:0;s:9:""permalink"";i:1;s:4:""type"";}}s:4:""page"";a:6:{s:3:""url"";s:12:""/[slug].html"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";s:4:""regx"";s:22:""|^/([^/]+)\.html[/]?$|"";s:6:""format"";s:8:""/%s.html"";s:6:""params"";a:1:{i:0;s:4:""slug"";}}}s:5:""index"";a:3:{s:3:""url"";s:1:""/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:7:""archive"";a:3:{s:3:""url"";s:6:""/blog/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:2:""do"";a:3:{s:3:""url"";s:22:""/action/[action:alpha]"";s:6:""widget"";s:14:""\Widget\Action"";s:6:""action"";s:6:""action"";}s:4:""post"";a:3:{s:3:""url"";s:24:""/archives/[cid:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:10:""attachment"";a:3:{s:3:""url"";s:26:""/attachment/[cid:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:8:""category"";a:3:{s:3:""url"";s:17:""/category/[slug]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:3:""tag"";a:3:{s:3:""url"";s:12:""/tag/[slug]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:6:""author"";a:3:{s:3:""url"";s:22:""/author/[uid:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:6:""search"";a:3:{s:3:""url"";s:19:""/search/[keywords]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:10:""index_page"";a:3:{s:3:""url"";s:21:""/page/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:12:""archive_page"";a:3:{s:3:""url"";s:26:""/blog/page/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:13:""category_page"";a:3:{s:3:""url"";s:32:""/category/[slug]/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:8:""tag_page"";a:3:{s:3:""url"";s:27:""/tag/[slug]/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:11:""author_page"";a:3:{s:3:""url"";s:37:""/author/[uid:digital]/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:11:""search_page"";a:3:{s:3:""url"";s:34:""/search/[keywords]/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:12:""archive_year"";a:3:{s:3:""url"";s:18:""/[year:digital:4]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:13:""archive_month"";a:3:{s:3:""url"";s:36:""/[year:digital:4]/[month:digital:2]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:11:""archive_day"";a:3:{s:3:""url"";s:52:""/[year:digital:4]/[month:digital:2]/[day:digital:2]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:17:""archive_year_page"";a:3:{s:3:""url"";s:38:""/[year:digital:4]/page/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:18:""archive_month_page"";a:3:{s:3:""url"";s:56:""/[year:digital:4]/[month:digital:2]/page/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:16:""archive_day_page"";a:3:{s:3:""url"";s:72:""/[year:digital:4]/[month:digital:2]/[day:digital:2]/page/[page:digital]/"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:12:""comment_page"";a:3:{s:3:""url"";s:53:""[permalink:string]/comment-page-[commentPage:digital]"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}s:4:""feed"";a:3:{s:3:""url"";s:20:""/feed[feed:string:0]"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:4:""feed"";}s:8:""feedback"";a:3:{s:3:""url"";s:31:""[permalink:string]/[type:alpha]"";s:6:""widget"";s:16:""\Widget\Feedback"";s:6:""action"";s:6:""action"";}s:4:""page"";a:3:{s:3:""url"";s:12:""/[slug].html"";s:6:""widget"";s:15:""\Widget\Archive"";s:6:""action"";s:6:""render"";}}"#,
        ],
        ["actionTable", "a:0:{}"],
        ["panelTable", "a:0:{}"],
        ["attachmentTypes", "@image@"],
        ["secret", &secret],
        ["installed", "1"],
        ["allowXmlRpc", "2"],
    ];
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            INSERT INTO {options_table} ("user", "name", "value")
            VALUES ('0', $1, $2)
            "#,
            options_table = &state.options_table,
        ),
        AnyKind::MySql => format!(
            r#"
            INSERT INTO {options_table} (`user`, `name`, `value`)
            VALUES ('0', ?, ?)
            "#,
            options_table = &state.options_table,
        ),
        _ => format!(
            r#"
            INSERT INTO {options_table} ("user", "name", "value")
            VALUES ('0', ?, ?)
            "#,
            options_table = &state.options_table,
        ),
    };
    for [name, value] in options{
        sqlx::query(&sql)
            .bind(name)
            .bind(value)
            .execute(&state.pool)
            .await
            .expect("options already exists");
    }
}
