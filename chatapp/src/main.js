import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import store from './store';
import './assets/styles.css'; // Import any global styles

const app = createApp(App);


store.dispatch('loadUserState');

app.use(store);
app.use(router);

app.mount('#app');
