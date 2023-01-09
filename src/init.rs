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
