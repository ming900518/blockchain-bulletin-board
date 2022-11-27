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
        // 資料還是需要存在，但不做序列化
        #[serde(skip_serializing)] T,
    ),
    // 無
    Empty,
}

impl<T> WithStatus<T> {
    // 把請求的String格式參數轉換成Enum
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
    // 確認請求的status參數是否正確
    fn check_status_string(str: &String) -> bool {
        str == "Open" || str == "Locked" || str == "Removed"
    }

    // 新增文章
    pub fn add_post(
        &mut self,
        title: String,
        content: String,
        tags: Vec<String>,
    ) -> WithStatus<Post> {
        // 產生新的文章
        let new_post = Post {
            id: self.number_of_posts,
            title,
            content,
            tags: tags.clone(),
            ..Post::default()
        };
        // 將新的文章存入
        self.posts.insert(&new_post.id, &Open(new_post.clone()));
        // 將請求中的tag存入
        tags.iter()
            .for_each(|new_tag| match self.tags.get(new_tag) {
                // 如果已經有tag存在，將id存入vector中
                Some(mut posts_id_vec) => posts_id_vec.push(new_post.id),
                // 如果tag不存在，建立一個新的vector，並將新的tag存入
                None => {
                    self.tags.insert(new_tag, &vec![new_post.id]);
                }
            });
        // post總數+1
        self.number_of_posts += 1;
        // 回傳
        Open(new_post)
    }

    // 查詢所有文章
    pub fn get_all_post(&self) -> Vec<(u128, WithStatus<Post>)> {
        self.posts
            .to_vec()
            .into_iter()
            // 僅有狀態為開放與鎖定的文章可以被查詢到
            .filter(|(_, post_with_status)| matches!(post_with_status, Open(_) | Locked(_)))
            .collect::<Vec<(u128, WithStatus<Post>)>>()
    }

    // 透過文字查詢文章
    pub fn search_post(&self, q: String) -> Vec<(u128, WithStatus<Post>)> {
        self.posts
            .to_vec()
            .into_iter()
            // 僅有狀態為開放與鎖定的文章可以被查詢到
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
            // 僅有狀態為開放與鎖定的文章可以被查詢到
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
            // 僅有狀態為開放與鎖定的文章可以被查詢到
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
                    // 僅有狀態為開放的文章可以被點讚
                    (true, Open(mut post)) => {
                        // 將自己的使用者ID存入
                        post.users_who_liked.push(env::signer_account_id());
                        // 儲存改好的文章
                        self.posts.insert(&id, &Open(post.clone()));
                        // 回傳文章
                        Open(post)
                    }
                    // 不符合條件，回傳找不到
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
                    // 僅有狀態為開放的文章可以被取消點讚
                    (true, Open(mut post)) => {
                        // 嘗試找出存放自己使用者ID的index
                        match &post
                            .users_who_liked
                            .clone()
                            .into_iter()
                            .position(|user_id| user_id == env::signer_account_id())
                        {
                            // 找不到自己的使用者ID，回傳
                            None => Empty,
                            // 找到使用者ID的index，繼續操作
                            Some(index) => {
                                // 將自己的使用者ID，透過index移除
                                post.users_who_liked.remove(*index);
                                // 儲存改好的文章
                                self.posts.insert(&id, &Open(post.clone()));
                                // 回傳文章
                                Open(post)
                            }
                        }
                    }
                    // 找不到文章，回傳
                    _ => Empty,
                },
            )
            // 只回傳一個（邏輯上一個post_id對應到的就只會有一個）
            .next()
            // 如果一個都找不到，就回傳無
            .unwrap_or(Empty)
    }

    // 編輯文章（只有原作者可以修改或移除文章）
    pub fn edit_post(
        &mut self,
        post_id: u128,
        title: Option<String>,
        content: Option<String>,
        tags: Option<Vec<String>>,
        status: String,
    ) -> WithStatus<Post> {
        // 嘗試找出文章
        match self.posts.get(&post_id) {
            // 找不到，回傳無
            None => Empty,
            // 找到，繼續操作
            Some(original_post_with_status) => {
                // 判斷找到的文章的狀態，以及確認status是否正確
                match (
                    original_post_with_status,
                    Self::check_status_string(&status),
                ) {
                    // 開放，可以做任何操作
                    (Open(original_post), true) => {
                        // 確認身份，只有原作者可以修改
                        if original_post.creator_user_id == env::signer_account_id() {
                            // 製作新的文章，有指定參數的才改
                            // 沒有指定參數的欄位（JSON填null），把原本文章的資訊填回去
                            let edited_post_with_status = WithStatus::new_with_status_string(
                                Post {
                                    title: title.unwrap_or(original_post.title),
                                    content: content.unwrap_or(original_post.content),
                                    tags: tags.unwrap_or(original_post.tags),
                                    ..original_post
                                },
                                status,
                            );
                            // 儲存修改過的文章
                            self.posts.insert(&post_id, &edited_post_with_status);
                            // 回傳
                            edited_post_with_status
                        } else {
                            // 身份不對，直接回傳無
                            Empty
                        }
                    }
                    // 鎖定，只能移除
                    (Locked(original_post), true) => {
                        // 確認身份，只有原作者可以修改 & 確認是要移除文章
                        if original_post.creator_user_id == env::signer_account_id()
                            && status == "Removed"
                        {
                            let edited_post_with_status =
                                WithStatus::new_with_status_string(original_post, status);
                            // 儲存修改過的文章
                            self.posts.insert(&post_id, &edited_post_with_status);
                            // 回傳
                            edited_post_with_status
                        } else {
                            // 身份不對或不是要移除文章，直接回傳無
                            Empty
                        }
                    }
                    // 其他狀態或者status有問題都不能異動
                    _ => Empty,
                }
            }
        }
    }
}
