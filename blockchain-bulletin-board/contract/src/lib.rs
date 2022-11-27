use crate::WithStatus::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId};

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde", tag = "status")]
pub enum WithStatus<T> {
    // 開放，全部人皆可查看，原作者可修改
    Open(T),
    // 鎖定，全部人皆可查看，不可修改
    Locked(T),
    // 移除，不可查看不可修改
    Removed(
        #[serde(skip_serializing)]
        T
    ),
    // 無
    Empty,
}

impl<T> WithStatus<T> {
    fn new_with_status_string(obj: T, str: String) -> WithStatus<T> {
        if str == "Open" {
            Open(obj)
        } else if str == "Locked" {
            Locked(obj)
        } else if str == "Removed" {
            Removed(obj)
        } else {
            Empty
        }
    }
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
    // 留言
    comments: Vec<WithStatus<Comment>>,
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
            comments: Vec::default(),
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
    // 子留言
    sub_comments: Vec<WithStatus<SubComment>>,
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
}

impl Default for Comment {
    fn default() -> Self {
        Self {
            comment_creator_user_id: env::signer_account_id(),
            content: String::default(),
            users_who_liked: Vec::default(),
            sub_comments: Vec::default(),
        }
    }
}

impl Default for SubComment {
    fn default() -> Self {
        Self {
            comment_creator_user_id: env::signer_account_id(),
            content: String::default(),
            users_who_liked: Vec::default(),
        }
    }
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct BulletinBoard {
    posts: UnorderedMap<u128, WithStatus<Post>>,
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
    pub fn add_post(
        &mut self,
        title: String,
        content: String,
        tags: Vec<String>,
    ) -> WithStatus<Post> {
        let new_post = Post {
            id: self.number_of_posts,
            title,
            content,
            tags: tags.clone(),
            ..Post::default()
        };

        self.posts.insert(&new_post.id, &Open(new_post.clone()));

        tags.iter()
            .for_each(|new_tag| match self.tags.get(new_tag) {
                Some(mut posts_id_vec) => posts_id_vec.push(new_post.id),
                None => {
                    self.tags.insert(new_tag, &vec![new_post.id]);
                }
            });

        self.number_of_posts += 1;
        Open(new_post)
    }

    // 查詢所有文章
    pub fn get_all_post(&self) -> Vec<(u128, WithStatus<Post>)> {
        self.posts.to_vec()
    }

    // 透過文字查詢文章
    pub fn search_post(&self, q: String) -> Vec<(u128, WithStatus<Post>)> {
        self.posts
            .to_vec()
            .into_iter()
            .filter(|(_, post_with_status)| match post_with_status {
                Open(post) | Locked(post) => post.title.contains(&q) || post.content.contains(&q),
                _ => false,
            })
            .collect::<Vec<(u128, WithStatus<Post>)>>()
    }

    // 透過標籤查詢文章
    pub fn search_post_by_tags(&self, tags: Vec<String>) -> Vec<(u128, WithStatus<Post>)> {
        self.posts
            .to_vec()
            .into_iter()
            .filter(|(_, post_with_status)| match post_with_status {
                Open(post) | Locked(post) => tags.iter().all(|tag| post.tags.contains(tag)),
                _ => false,
            })
            .collect::<Vec<(u128, WithStatus<Post>)>>()
    }

    // 透過使用者ID查詢文章
    pub fn search_post_by_user_id(
        &self,
        creator_user_id: AccountId,
    ) -> Vec<(u128, WithStatus<Post>)> {
        self.posts
            .to_vec()
            .into_iter()
            .filter(|(_, post_with_status)| match post_with_status {
                Open(post) | Locked(post) => creator_user_id == post.creator_user_id,
                _ => false,
            })
            .collect::<Vec<(u128, WithStatus<Post>)>>()
    }

    // 點讚
    pub fn like_post(&mut self, post_id: u128) -> WithStatus<Post> {
        self.posts
            .to_vec()
            .into_iter()
            .map(
                |(id, post_with_status)| match (id == post_id, post_with_status) {
                    (true, Open(mut post)) => {
                        post.users_who_liked.push(env::signer_account_id());
                        self.posts.insert(&id, &Open(post.clone()));
                        Open(post)
                    }
                    _ => Empty,
                },
            )
            .next()
            .unwrap_or(Empty)
    }

    // 取消點讚
    pub fn unlike_post(&mut self, post_id: u128) -> WithStatus<Post> {
        self.posts
            .to_vec()
            .into_iter()
            .map(
                |(id, post_with_status)| match (id == post_id, post_with_status) {
                    (true, Open(mut post)) => {
                        match &post
                            .users_who_liked
                            .clone()
                            .into_iter()
                            .position(|user_id| user_id == env::signer_account_id())
                        {
                            None => Empty,
                            Some(index) => {
                                post.users_who_liked.remove(*index);
                                self.posts.insert(&id, &Open(post.clone()));
                                Open(post)
                            }
                        }
                    }
                    _ => Empty,
                },
            )
            .next()
            .unwrap_or(Empty)
    }

    // 編輯文章（只有原作者可以修改或移除文章）
    pub fn edit_post(
        &mut self,
        post_id: u128,
        title: Option<String>,
        content: Option<String>,
        tags: Option<Vec<String>>,
        status: String
    ) -> WithStatus<Post> {
        match self.posts.get(&post_id) {
            None => Empty,
            Some(original_post_with_status) => {
                match original_post_with_status {
                    Open(original_post) => {
                        if original_post.creator_user_id == env::signer_account_id() {
                            let edited_post_with_status = &WithStatus::new_with_status_string(Post {
                                title: title.unwrap_or(original_post.title),
                                content: content.unwrap_or(original_post.content),
                                tags: tags.unwrap_or(original_post.tags),
                                ..original_post
                            }, status);
                            match edited_post_with_status{
                                Empty => Empty,
                                _ => {
                                    self.posts.insert(&post_id, edited_post_with_status);
                                    self.posts.get(&post_id).unwrap_or(Empty)
                                }
                            }
                        } else {
                            Empty
                        }
                    },
                    Locked(original_post) => {
                        if original_post.creator_user_id == env::signer_account_id() && status == "Removed" {
                            self.posts.insert(&post_id, &WithStatus::new_with_status_string(original_post, status));
                            self.posts.get(&post_id).unwrap_or(Empty)
                        } else {
                            Empty
                        }
                    },
                    _ => Empty
                }
            }
        }
    }
}
