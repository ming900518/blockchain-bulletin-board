use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId};

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde", tag = "type")]
pub enum PostStatus {
    // 開放，全部人皆可查看，原作者可修改
    Open,
    // 鎖定，全部人皆可查看，原作者不可修改
    Locked,
    // 移除，不可查看不可修改
    Removed
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
    status: PostStatus
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
            users_who_liked: Vec::<AccountId>::default(),
            creator_user_id: env::signer_account_id(),
            status: PostStatus::Open,
        };

        self.posts.insert(&new_post.id, &new_post);

        tags.iter()
            .for_each(|new_tag| match self.tags.get(new_tag) {
                Some(mut posts_id_vec) => posts_id_vec.push(new_post.id),
                None => {
                    self.tags.insert(new_tag, &vec![new_post.id]);
                    ()
                }
            });

        self.number_of_posts += 1;
        new_post
    }

    // 查詢所有文章
    pub fn get_all_post(&self) -> Vec<(u128, Post)> {
        Self::get_not_removed_post_vec(&self.posts)
    }

    // 透過文字查詢文章
    pub fn search_post(&self, q: String) -> Vec<(u128, Post)> {
        Self::get_not_removed_post_vec(&self.posts)
            .into_iter()
            .filter(|(_, post)| post.title.contains(&q) || post.content.contains(&q))
            .collect::<Vec<(u128, Post)>>()
    }

    // 透過標籤查詢文章
    pub fn search_post_by_tags(&self, tags: Vec<String>) -> Vec<(u128, Post)> {
        Self::get_not_removed_post_vec(&self.posts)
            .into_iter()
            .filter(|(_, post)| tags.iter().all(|tag| post.tags.contains(tag)))
            .collect::<Vec<(u128, Post)>>()
    }

    // 透過使用者ID查詢文章
    pub fn search_post_by_user_id(&self, creator_user_id: AccountId) -> Vec<(u128, Post)> {
        Self::get_not_removed_post_vec(&self.posts)
            .into_iter()
            .filter(|(_, post)| post.creator_user_id == creator_user_id)
            .collect::<Vec<(u128, Post)>>()
    }

    // 查詢所有沒有被移除的文章（內部用）
    fn get_not_removed_post_vec(posts: &UnorderedMap<u128, Post>) -> Vec<(u128, Post)> {
        posts.to_vec()
            .into_iter()
            .filter(|(_, post)| post.status != PostStatus::Removed)
            .collect::<Vec<(u128, Post)>>()
    }
}
