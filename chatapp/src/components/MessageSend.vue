<template>
  <div class="message-send">
    <div class="upload-container">
      <input
        type="file"
        @change="handleFileUpload"
        ref="fileInput"
        multiple
        accept="image/*"
        style="display: none"
      />
      <button @click="triggerFileInput" class="upload-button">
        <svg xmlns="http://www.w3.org/2000/svg" class="icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
      </button>
    </div>
    <div class="input-container">
      <input
        v-model="message"
        @keyup.enter="sendMessage"
        placeholder="输入消息..."
        class="message-input"
        type="text"
      />
      <div class="image-preview">
        <img v-for="(file, index) in files" :key="index" :src="file.fullUrl" class="thumbnail" />
      </div>
    </div>
    <button @click="sendMessage" class="send-button">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        class="icon"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M5 12h14M12 5l7 7-7 7"
        />
      </svg>
    </button>
  </div>
</template>

<script>
export default {
  data() {
    return {
      message: '',
      files: [],
    };
  },
  computed: {
    userId() {
      return this.$store.state.user.id;
    },
    workspaceId() {
      return this.$store.state.workspace.id;
    },
    activeChannelId() {
      let channel = this.$store.state.activeChannel;
      if (!channel) {
        return null;
      }
      return channel.id;
    },
  },
  methods: {
    triggerFileInput() {
      this.$refs.fileInput.click();
    },
    async handleFileUpload(event) {
      const files = Array.from(event.target.files);
      if (files.length > 0) {
        try {
          const uploadedFiles = await this.$store.dispatch('uploadFiles', files);
          this.files = uploadedFiles;
        } catch (error) {
          console.error('上传图片失败:', error);
        }
      }
    },
    async sendMessage() {
      if (this.message.trim() === '') {
        return;
      }

      const payload = {
        chatId: this.activeChannelId,
        content: this.message,
        files: this.files.map(file => file.path),
      };
      try {
        await this.$store.dispatch('sendMessage', payload);
        this.message = '';
        this.files = [];
      } catch (error) {
        console.error('发送消息失败:', error);
      }
    },
  },
};
</script>

<style scoped>
.message-send {
  display: flex;
  align-items: flex-end;
  padding: 10px 0px;
  background-color: #fff;
  border-top: 1px solid #eee;
}
.upload-container {
  margin-right: 10px;
}
.upload-button {
  background-color: #eee;
  border: none;
  border-radius: 50%;
  padding: 10px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}
.upload-button .icon {
  width: 20px;
  height: 20px;
}
.input-container {
  flex: 1;
  display: flex;
  flex-direction: column;
}
.message-input {
  width: 100%;
  padding: 12px 12px;
  border: none;
  border-radius: 10px;
  background-color: #eee;
  font-size: 14px;
}
.message-input::placeholder {
  color: #72767d;
}
.image-preview {
  display: flex;
  flex-wrap: wrap;
  margin-top: 5px;
}
.thumbnail {
  width: 50px;
  height: 50px;
  object-fit: cover;
  margin-right: 5px;
  margin-bottom: 5px;
  border-radius: 5px;
}
.send-button {
  background-color: #5865f2;
  border: none;
  border-radius: 50%;
  padding: 10px;
  margin-left: 10px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}
.send-button .icon {
  width: 20px;
  height: 20px;
}
.send-button:hover {
  background-color: #4752c4;
}
</style>
