# PromptOps Arena

**PromptOps Arena** 是一款以 AI Agent 行為設計為核心的多人小隊對戰遊戲。

玩家不直接操作角色，而是為自己的 Agent 撰寫行為 prompt。比賽開始後，每個 Agent 會根據自己的角色定位、目前戰場狀態、遊戲規則與玩家撰寫的 prompt 自主做出行動。隊伍需要透過討論、分工與策略設計，讓多個 Agent 在沒有即時人工操作的情況下形成有效配合。

這個專案希望把 Prompt Engineering 從單純的文字指令練習，轉化成一個可以觀察、比較、討論與反覆修正的遊戲體驗。

---

## Project Goal

PromptOps Arena 的核心目標是建立一個可遊玩的 AI Agent 團隊戰術遊戲，讓玩家能夠：

* 設計不同角色的 Agent 行為規則
* 觀察 prompt 如何影響 Agent 的實際決策
* 透過隊伍合作制定整體戰術
* 從比賽紀錄中找出 prompt 與預期行為的落差
* 反覆修改 prompt，讓 Agent 行為更穩定、更有策略性

這不是一款傳統即時操作遊戲。遊戲的重點不是手速或即時反應，而是玩家如何設計清楚、可執行、可調整的 Agent 行為邏輯。

---

## Core Gameplay

PromptOps Arena 是一款回合制格子戰術遊戲。

每場遊戲由兩隊對戰。每位玩家負責一名 Agent，並在遊戲開始前為該 Agent 撰寫 prompt。當遊戲開始後，Agent 會依照 prompt 與戰場資訊自行決定每回合的移動與行動。

每個 Agent 每回合通常可以：

1. 移動一次
2. 執行一個主要行動

主要行動可能包含：

* 攻擊
* 使用技能
* 防守
* 佔領或強化目標點
* 等待或採取保守行動

遊戲的核心循環是：

1. 理解地圖、角色與勝利條件
2. 和隊友討論整體戰術
3. 為自己的 Agent 撰寫 prompt
4. 讓 Agent 根據 prompt 自動對戰
5. 觀察 Agent 行為與預期的差異
6. 修改 prompt 並再次驗證

---

## Game Objective

遊戲的主要勝利條件是控制地圖上的資料節點，而不是單純擊倒敵人。

地圖上有多個可爭奪的目標點。隊伍需要讓自己的 Agent 佔領、守住或干擾這些節點，藉此累積分數。擊倒敵方 Agent 可以提供額外優勢，但不應該取代地圖目標本身。

這樣的設計讓不同角色都能發揮價值：

* 前排角色可以推進與承受傷害
* 輸出角色可以壓制敵人
* 醫護角色可以維持隊伍續戰力
* 防守角色可以穩定守點
* 偵查角色可以提供資訊與側路壓力
* 工程角色可以經營節點與控制區域

勝利來自整體策略，而不是單一角色的輸出。

---

## Agents and Roles

每隊由多名不同角色的 Agent 組成。每個角色都有自己的定位、能力與戰術價值。

目前的核心角色包含：

| Role     | 中文名稱 | Core Responsibility |
| -------- | ---- | ------------------- |
| Vanguard | 前鋒   | 推進、承傷、進入關鍵節點        |
| Striker  | 攻擊手  | 遠程輸出、壓制與收割          |
| Medic    | 醫護   | 治療、復活、維持隊伍續戰力       |
| Guardian | 守衛   | 守點、防線、保護隊友          |
| Scout    | 偵查   | 探路、標記、側路牽制          |
| Engineer | 工程師  | 佔點、強化節點、區域控制        |

每個玩家負責其中一名 Agent。好的 prompt 不只要描述「要做什麼」，還需要包含目標、條件、優先順序、撤退規則與 fallback 行動。

---

## Prompt as Gameplay

在 PromptOps Arena 中，prompt 本身就是遊戲機制的一部分。

一個好的 Agent prompt 應該能回答：

* 這個角色的主要任務是什麼？
* 遇到多個目標時，應該優先處理哪一個？
* 什麼時候應該進攻？
* 什麼時候應該撤退？
* 什麼情況下應該保護隊友？
* 什麼情況下應該優先佔點？
* 資訊不足時應該採取什麼保守行動？

玩家需要從 Agent 的實際行為中判斷 prompt 是否足夠清楚。如果 Agent 做出奇怪的決策，問題可能不是 Agent 不聰明，而是 prompt 沒有給出足夠明確的條件、優先順序或 fallback。

---

## Map and Match Design

遊戲採用對稱式格子地圖，讓兩隊在公平條件下進行戰術對抗。

地圖設計重點包含：

* 左右對稱，確保雙方起始條件公平
* 多條路線，讓隊伍可以選擇集中推進或分路牽制
* 多個資料節點，鼓勵地圖控制與角色分工
* 牆壁與掩體，讓站位、視野與攻擊路線變得重要
* 適中的地圖大小，讓對戰能在短時間內產生有效互動

遊戲鼓勵玩家思考「角色應該如何配合地圖目標」，而不是只讓所有 Agent 追著敵人攻擊。

---

## Technical Overview

PromptOps Arena 是一個 web-based multiplayer game project。

目前技術方向包含：

* **Rust**：核心遊戲邏輯與伺服器實作
* **Tokio**：非同步 runtime
* **axum**：Web framework
* **WebSocket**：即時房間狀態、遊戲控制與觀戰更新
* **Serde**：資料序列化與反序列化
* **In-memory persistence**：開發階段的資料儲存層
* **Replaceable persistence layer**：未來可替換成 MongoDB 或其他資料庫
* **TypeScript**：Web client 與共享型別的可能實作方向
* **Monorepo**：集中管理遊戲相關程式碼、共享型別、工具與文件

技術設計的核心原則是：

* 遊戲規則應該可測試、可重播、可除錯
* Match simulation 應該盡可能 deterministic
* Agent 的行動必須由遊戲規則驗證，而不是直接相信模型輸出
* WebSocket 是即時互動的主要通訊方式
* Persistence 應該透過抽象層存取，而不是綁定特定資料庫
* 專案應該能從本地 memory-based MVP 演進到可持久化部署

---

## Project Status

This project is under active development.

Current focus:

* 建立核心遊戲規則
* 設計房間與對戰流程
* 建立可測試的 match simulation
* 實作即時多人互動
* 支援 Agent prompt 驅動的對戰流程
* 產生可回放、可分析的遊戲紀錄

---

## Development Philosophy

PromptOps Arena 的開發重點不是一次做出複雜的戰棋系統，而是先建立一個穩定、可測試、可觀察的核心遊戲循環。

優先順序：

1. 規則簡單但有策略深度
2. Agent 行為可觀察
3. Prompt 修改能帶來明確差異
4. 對戰流程穩定
5. 遊戲紀錄可重播
6. 系統架構能逐步擴充

不必要的複雜度應該延後處理。遊戲的核心價值在於讓玩家看見 prompt 如何影響 Agent 行為，而不是堆疊過多規則、技能或隨機性。

---

## License

MIT
