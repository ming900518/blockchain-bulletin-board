# blockchain-bulletin-board
A bulletin board system built with Rust and NEAR smart contract.

## 說明
### 目標
利用Rust與智能合約，實作BBS（電子佈告欄系統）

### 採用技術
| 項目   | 框架/語言 |
|------|-------|
| 程式語言 | Rust  |
| 區塊鏈  | NEAR  |

### 功能
- [x] BBS與文章的結構
- [x] 新增文章
- [x] 查詢文章
- [x] 更新文章
- [x] 移除文章
- [x] 文章點讚/移除讚
- [x] 新增留言/子留言
- [x] 更新留言/子留言
- [ ] 移除留言/子留言
- [ ] 留言/子留言點讚/移除讚
- [ ] 留言/子留言置頂
- [ ] 前端畫面

> 智能合約並沒有「移除」的概念，利用文章/留言/子留言結構中的status判斷前端是否顯示
