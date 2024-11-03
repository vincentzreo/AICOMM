<template>
  <div class="message-list">
    <div v-if="messages.length === 0" class="no-messages">
      该频道暂无消息。
    </div>
    <div v-else>
      <div v-for="message in reversedMessages" :key="message.id" class="message">
        <img :src="`https://ui-avatars.com/api/?name=${getSender(message.senderId).fullname.replace(' ', '+')}`" class="avatar" alt="头像" />
        <div class="message-content">
          <div class="message-header">
            <span class="message-user">{{ getSender(message.senderId).fullname }}</span>
            <span class="message-time">{{ formatTime(message.createdAt) }}</span>
          </div>
          <div class="message-text">{{ getMessageContent(message) }}</div>
          <div v-if="message.files && message.files.length > 0" class="message-images">
            <div v-for="(file, index) in message.files" :key="index" class="image-container">
              <img :src="getFileUrl(file)"
                   :class="{'thumbnail': true, 'active': isActive(message.id, index)}"
                   @click="toggleImageSize(message.id, index, file)"
                   :alt="'附件图片 ' + (index + 1)" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
  <!-- 添加一个遮罩层和大图容器 -->
  <div v-if="largeImage" class="overlay" @click="closeLargeImage">
    <img :src="getFileUrl(largeImage)" class="large-image" @click.stop />
  </div>
</template>

<script>
import { getUrlBase } from '../util';
import { ref } from 'vue';  // 导入 ref

export default {
  setup() {
    const largeImage = ref(null);
    const activeImageKey = ref(null);

    return {
      largeImage,
      activeImageKey
    };
  },
  computed: {
    messages() {
      return this.$store.getters.getMessagesForActiveChannel;
    },
    reversedMessages() {
      return [...this.messages].reverse();
    },
    activeChannelId() {
      let channel = this.$store.state.activeChannel;
      if (!channel) {
        return null;
      }
      return channel.id;
    }
  },
  watch: {
    activeChannelId(newChannelId) {
      if (newChannelId) {
        this.fetchMessages(newChannelId);
      }
    }
  },
  methods: {
    formatTime(time) {
      const date = new Date(time);
      return date.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit'
      });
    },
    fetchMessages(channelId) {
      this.$store.dispatch('fetchMessagesForChannel', channelId);
    },
    getSender(userId) {
      return this.$store.getters.getUserById(userId);
    },
    isActive(messageId, index) {
      return this.activeImageKey === `${messageId}-${index}`;
    },
    toggleImageSize(messageId, index, file) {
      const key = `${messageId}-${index}`;
      if (this.activeImageKey === key) {
        this.closeLargeImage();
      } else {
        this.activeImageKey = key;
        this.largeImage = file;
      }
    },
    closeLargeImage() {
      this.activeImageKey = null;
      this.largeImage = null;
    },
    getFileUrl(file) {
      return `${getUrlBase()}${file}?token=${this.$store.state.token}`;
    },
    getMessageContent(message) {
      if (message.senderId === this.$store.state.user.id) {
        return message.content;
      } else {
        return message.modifiedContent && message.modifiedContent.trim() !== ''
          ? message.modifiedContent
          : message.content;
      }
    }
  },
  mounted() {
    if (this.activeChannelId) {
      this.fetchMessages(this.activeChannelId);
    }
  }
};
</script>

<style scoped>
/* 容器样式 */
.message-list {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column-reverse;
  padding-bottom: 120px;
}
/* 单个消息样式 */
.message {
  display: flex;
  align-items: flex-start;
  margin-bottom: 20px;
}
.avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  margin-right: 10px;
}
.message-content {
  max-width: 80%;
}
/* 头部样式：用户名和时间戳 */
.message-header {
  display: flex;
  align-items: center;
  margin-bottom: 5px;
}
.message-user {
  font-weight: bold;
  margin-right: 10px;
}
.message-time {
  font-size: 12px;
}
/* 消息文本样式 */
.message-text {
  font-size: 14px;
  line-height: 1.4;
  word-wrap: break-word;
  white-space: pre-wrap;
}

.no-messages {
  text-align: center;
  color: #b9bbbe;
  margin-top: 20px;
}

/* 图片网格样式 */
.message-images {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
  gap: 10px;
  margin-top: 10px;
}

.image-container {
  width: 100%;
  height: 100px;
  overflow: hidden;
}

.thumbnail {
  width: 100%;
  height: 100%;
  object-fit: cover;
  cursor: pointer;
  transition: transform 0.3s ease;
}

.large {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  max-width: 90vw;
  max-height: 90vh;
  object-fit: contain;
  z-index: 1000;
  cursor: pointer;
}

.thumbnail:hover {
  transform: scale(1.05);
}

.overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.8);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.large-image {
  max-width: 90vw;
  max-height: 90vh;
  object-fit: contain;
}

.thumbnail.active {
  border: 2px solid #5865f2;
}
</style>
