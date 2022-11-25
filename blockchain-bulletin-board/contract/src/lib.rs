use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId};
use std::collections::VecDeque;

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde", tag = "type")]
pub enum Status {
    // 開放，全部人皆可查看，原作者可修改
    Open,
    // 鎖定，全部人皆可查看，原作者不可修改
    Locked,
    // 移除，不可查看不可修改
    Removed,
    // 不存在
    NotFound,
    // 無權限
    NoPermission,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Post {
    // 文章流水號
    id: u128,
    // 標題
    title: String,
    // 內文
    content: String,
    // 標籤
    tags: Vec<String>,
    // 點讚用戶
    users_who_liked: Vec<AccountId>,
    // 作者
    creator_user_id: AccountId,
    // 文章狀態
    status: Status,
    // 留言（利用VecDeque可以更有效率的實作留言置頂功能，詳情見https://doc.rust-lang.org/std/collections/struct.VecDeque.html）
    comments: VecDeque<Comment>,
}

impl Default for Post {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::default(),
            content: String::default(),
            tags: Vec::default(),
            users_who_liked: Vec::default(),
            creator_user_id: env::signer_account_id(),
            status: Status::Open,
            comments: VecDeque::default(),
        }
    }
}

impl Post {
    fn not_found() -> Self {
        Post {
            id: u128::MAX,
            status: Status::NotFound,
            ..Post::default()
        }
    }

    fn no_permission() -> Self {
        Post {
            id: u128::MAX,
            status: Status::NoPermission,
            ..Post::default()
        }
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
// 留言
pub struct Comment {
    // 留言者
    comment_creator_user_id: AccountId,
    // 內容
    content: String,
    // 點讚用戶
    users_who_liked: Vec<AccountId>,
    // 留言狀態
    status: Status,
    // 子留言（利用VecDeque可以更有效率的實作留言置頂功能，詳情見https://doc.rust-lang.org/std/collections/struct.VecDeque.html）
    sub_comment: VecDeque<SubComment>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
// 子留言
pub struct SubComment {
    // 留言者
    comment_creator_user_id: AccountId,
    // 內容
    content: String,
    // 點讚用戶
    users_who_liked: Vec<AccountId>,
    // 子留言狀態
    status: Status,
}

impl Default for Comment {
    fn default() -> Self {
        Self {
            comment_creator_user_id: env::signer_account_id(),
            content: String::default(),
            users_who_liked: Vec::default(),
            status: Status::Open,
            sub_comment: VecDeque::default(),
        }
    }
}

impl Default for SubComment {
    fn default() -> Self {
        Self {
            comment_creator_user_id: env::signer_account_id(),
            content: String::default(),
            users_who_liked: Vec::default(),
            status: Status::Open,
        }
    }
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct BulletinBoard {
    posts: UnorderedMap<u128, Post>,
    tags: UnorderedMap<String, Vec<u128>>,
    number_of_posts: u128,
    likes_by_user_id: UnorderedMap<AccountId, Vec<Post>>,
}

impl Default for BulletinBoard {
    fn default() -> Self {
        Self {
            posts: UnorderedMap::new(b'm'),
            tags: UnorderedMap::new(b'n'),
            number_of_posts: 0,
            likes_by_user_id: UnorderedMap::new(b'o'),
        }
    }
}

#[near_bindgen]
impl BulletinBoard {
    // 新增文章
    pub fn add_post(&mut self, title: String, content: String, tags: Vec<String>) -> Post {
        let new_post = Post {
            id: self.number_of_posts,
            title,
            content,
            tags: tags.clone(),
            ..Post::default()
        };

        self.posts.insert(&new_post.id, &new_post);

        tags.iter()
            .for_each(|new_tag| match self.tags.get(new_tag) {
                Some(mut posts_id_vec) => posts_id_vec.push(new_post.id),
                None => {
                    self.tags.insert(new_tag, &vec![new_post.id]);
                }
            });

        self.number_of_posts += 1;
        new_post
    }

    // 查詢所有文章
    pub fn get_all_post(&self) -> Vec<(u128, Post)> {
        Self::get_post_vec(&self.posts)
    }

    // 透過文字查詢文章
    pub fn search_post(&self, q: String) -> Vec<(u128, Post)> {
        Self::get_post_vec(&self.posts)
            .into_iter()
            .filter(|(_, post)| post.title.contains(&q) || post.content.contains(&q))
            .collect::<Vec<(u128, Post)>>()
    }

    // 透過標籤查詢文章
    pub fn search_post_by_tags(&self, tags: Vec<String>) -> Vec<(u128, Post)> {
        Self::get_post_vec(&self.posts)
            .into_iter()
            .filter(|(_, post)| tags.iter().all(|tag| post.tags.contains(tag)))
            .collect::<Vec<(u128, Post)>>()
    }

    // 透過使用者ID查詢文章
    pub fn search_post_by_user_id(&self, creator_user_id: AccountId) -> Vec<(u128, Post)> {
        Self::get_post_vec(&self.posts)
            .into_iter()
            .filter(|(_, post)| post.creator_user_id == creator_user_id)
            .collect::<Vec<(u128, Post)>>()
    }

    // 點讚
    pub fn like_post(&mut self, post_id: u128) -> Post {
        match Self::get_post(&self.posts, &post_id) {
            None => Post::not_found(),
            Some((id, mut post)) => {
                post.users_who_liked.push(env::signer_account_id());
                self.posts.insert(&id, &post);
                post
            }
        }
    }

    // 取消點讚
    pub fn dislike_post(&mut self, post_id: u128) -> Post {
        match Self::get_post(&self.posts, &post_id) {
            None => Post::not_found(),
            Some((id, mut post)) => {
                let users_who_liked = &mut post.users_who_liked;
                match users_who_liked
                    .iter_mut()
                    .position(|user_id| user_id == &env::signer_account_id())
                {
                    None => Post::not_found(),
                    Some(position) => {
                        users_who_liked.remove(position);
                        self.posts.insert(&id, &post);
                        post
                    }
                }
            }
        }
    }

    // 編輯文章（只有原作者可以修改或移除文章）
    pub fn edit_post(
        &mut self,
        post_id: u128,
        title: String,
        content: String,
        tags: Vec<String>,
        status: Status,
    ) -> Post {
        match Self::get_post(&self.posts, &post_id) {
            None => Post::not_found(),
            Some((id, mut post)) => {
                match (
                    post.creator_user_id == env::signer_account_id(),
                    &post.status,
                    &status,
                ) {
                    // 如果是原作者，且文章處於開放狀態
                    (true, Status::Open, _) => {
                        match status {
                            // 如果要把文章鎖定或移除，忽略其他參數，直接改狀態
                            Status::Locked | Status::Removed => post.status = status,
                            // 其他情況則需要儲存參數中的內容
                            _ => {
                                post.title = title;
                                post.content = content;
                                post.tags = tags;
                            }
                        }
                        self.posts.insert(&id, &post);
                        post
                    }
                    // 如果是原作者，且文章處於鎖定狀態，原作者想要刪除文章
                    (true, Status::Locked, Status::Removed) => {
                        post.status = status;
                        self.posts.insert(&id, &post);
                        post
                    }
                    (_, _, _) => Post::no_permission(),
                }
            }
        }
    }

    // 新增留言
    pub fn add_comment(
        &mut self,
        post_id: u128,
        comment_index: Option<u128>,
        content: String,
    ) -> Post {
        match Self::get_post(&self.posts, &post_id) {
            None => Post::not_found(),
            Some((id, mut post)) => {
                // 如果有指定comment_index，代表要新增的是子留言
                match comment_index {
                    None => {
                        // 直接做一個新的留言塞進文章留言的最後面
                        post.comments.push_back(Comment {
                            content,
                            ..Comment::default()
                        });
                    }
                    Some(index) => {
                        // 判斷留言是否存在
                        match post.comments.get(index as usize) {
                            // 不存在就不做任何處理
                            None => (),
                            // 存在
                            Some(comment) => {
                                // 複製一份原有留言
                                let mut new_comment = comment.clone();
                                // 把新的子留言塞到子留言的最後面
                                new_comment.sub_comment.push_back(SubComment {
                                    content,
                                    ..SubComment::default()
                                });
                                // 刪除原有留言
                                post.comments.remove(index as usize);
                                // 把新的留言塞進文章留言的最後面
                                post.comments.push_back(new_comment);
                            }
                        }
                    }
                };
                self.posts.insert(&id, &post);
                post
            }
        }
    }

    // 編輯留言
    pub fn edit_comment(
        &mut self,
        post_id: u128,
        comment_index: u128,
        sub_comment_index: Option<u128>,
        content: String,
        status: Status,
    ) -> Post {
        match Self::get_post(&self.posts, &post_id) {
            None => Post::not_found(),
            Some((id, mut post)) => {
                // 把文章中的留言撈出來
                match post.comments.get_mut(comment_index as usize) {
                    // 沒有就忽略
                    None => (),
                    // 有留言
                    Some(comment) => {
                        // 判斷是否有指定sub_comment_index
                        match sub_comment_index {
                            // 沒指定，直接寫入原有留言
                            None => {
                                // 判斷是否為原作者&留言狀態
                                match (
                                    comment.comment_creator_user_id == env::signer_account_id(),
                                    &comment.status,
                                    &status,
                                ) {
                                    // 是原作者&留言開放
                                    (true, Status::Open, _) => {
                                        comment.content = content;
                                        comment.status = status;
                                    }
                                    // 是原作者&留言鎖定，原作者要刪除
                                    (true, Status::Locked, Status::Removed) => {
                                        comment.status = status;
                                    }
                                    (_, _, _) => (),
                                }
                            }
                            // 有指定
                            Some(index) => {
                                // 撈出子留言
                                match comment.sub_comment.get_mut(index as usize) {
                                    // 沒有就忽略
                                    None => (),
                                    // 有子留言
                                    Some(sub_comment) => {
                                        // 判斷是否為原作者&留言狀態
                                        match (
                                            sub_comment.comment_creator_user_id
                                                == env::signer_account_id(),
                                            &sub_comment.status,
                                            &status,
                                        ) {
                                            // 是原作者&留言開放
                                            (true, Status::Open, _) => {
                                                sub_comment.content = content;
                                                sub_comment.status = status;
                                            }
                                            // 是原作者&留言鎖定，原作者要刪除
                                            (true, Status::Locked, Status::Removed) => {
                                                sub_comment.status = status;
                                            }
                                            (_, _, _) => (),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                self.posts.insert(&id, &post);
                post
            }
        }
    }

    // 查詢所有沒有被移除的文章（內部用）
    fn get_post_vec(posts: &UnorderedMap<u128, Post>) -> Vec<(u128, Post)> {
        posts
            .to_vec()
            .into_iter()
            // 把移除的文章過濾掉
            .filter(|(_, post)| post.status != Status::Removed)
            .map(|(id, mut post)| {
                let filtered_comment = post
                    .comments
                    .into_iter()
                    // 把移除的留言過濾掉
                    .filter(|comments| comments.status != Status::Removed)
                    .map(|mut comments| {
                        comments.sub_comment = comments
                            .sub_comment
                            .into_iter()
                            // 把移除的子留言過濾掉
                            .filter(|sub_comment| sub_comment.status != Status::Removed)
                            .collect::<VecDeque<SubComment>>();
                        comments
                    })
                    .collect::<VecDeque<Comment>>();
                post.comments = filtered_comment;
                (id, post)
            })
            .collect::<Vec<(u128, Post)>>()
    }

    // 查詢指定的文章（內部用）
    fn get_post(posts: &UnorderedMap<u128, Post>, post_id: &u128) -> Option<(u128, Post)> {
        Self::get_post_vec(posts)
            .into_iter()
            .find(|(id, _)| id == post_id)
    }
}
