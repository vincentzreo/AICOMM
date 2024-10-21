<template>
  <div class="chat">
    <Sidebar />
    <div class="chat-main">
      <MessageList />
      <MessageSend />
    </div>
  </div>
</template>

<script>
import Sidebar from '../components/Sidebar.vue';
import MessageList from '../components/MessageList.vue';
import MessageSend from '../components/MessageSend.vue';
export default {
  components: {
    Sidebar,
    MessageList,
    MessageSend,
  },
};
</script>

<style scoped>
.chat {
  display: flex;
  height: 100vh; /* 全屏高度 */
}
.chat-main {
  display: flex;
  flex-direction: column;
  flex: 1;
  position: relative;
}
.chat-main > .message-list {
  flex: 1;
  overflow-y: auto; /* 可滚动的消息列表 */
  padding-bottom: 200px; /* 为输入框留出空间 */
}
.chat-main > .message-send {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  background-color: #fff; /* 确保输入框背景不透明 */
  z-index: 1; /* 确保输入框在消息列表上方 */
}
</style>
