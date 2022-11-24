use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId};

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde", tag = "type")]
pub enum PostStatus {
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
    status: PostStatus,
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
            status: PostStatus::Open,
        }
    }
}

impl Post {
    fn not_found() -> Self {
        Post {
            id: u128::MAX,
            status: PostStatus::NotFound,
            ..Post::default()
        }
    }

    fn no_permission() -> Self {
        Post {
            id: u128::MAX,
            status: PostStatus::NoPermission,
            ..Post::default()
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
        status: PostStatus,
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
                    (true, PostStatus::Open, _) => {
                        match status {
                            // 如果要把文章鎖定或移除，忽略其他參數，直接改狀態
                            PostStatus::Locked | PostStatus::Removed => post.status = status,
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
                    (true, PostStatus::Locked, PostStatus::Removed) => {
                        post.status = status;
                        self.posts.insert(&id, &post);
                        post
                    }
                    (_, _, _) => Post::no_permission(),
                }
            }
        }
    }

    // 查詢所有沒有被移除的文章（內部用）
    fn get_not_removed_post_vec(posts: &UnorderedMap<u128, Post>) -> Vec<(u128, Post)> {
        posts
            .to_vec()
            .into_iter()
            .filter(|(_, post)| post.status != PostStatus::Removed)
            .collect::<Vec<(u128, Post)>>()
    }

    // 查詢指定的文章（內部用）
    fn get_post(posts: &UnorderedMap<u128, Post>, post_id: &u128) -> Option<(u128, Post)> {
        posts
            .to_vec()
            .into_iter()
            .find(|(id, post)| post.status != PostStatus::Removed && id == post_id)
    }
}
