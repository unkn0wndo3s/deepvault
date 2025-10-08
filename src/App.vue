<template>
  <div id="app">
    <header class="app-header">
      <h1>DeepVault</h1>
      <p>Chiffrement sécurisé de périphériques USB</p>
      <nav class="app-nav">
        <button
          @click="currentView = 'devices'"
          :class="{ active: currentView === 'devices' }"
          class="nav-button"
        >
          🔍 Détecter USB
        </button>
        <button
          @click="currentView = 'explorer'"
          :class="{ active: currentView === 'explorer' }"
          class="nav-button"
        >
          📁 Explorateur
        </button>
      </nav>
    </header>

    <main class="app-main">
      <DeviceList
        v-if="currentView === 'devices'"
        @open-partition="handleOpenPartition"
      />
      <FileExplorer
        v-if="currentView === 'explorer'"
        :partition-info="selectedPartition"
        @close-explorer="handleCloseExplorer"
      />
    </main>

    <footer class="app-footer">
      <p>Version {{ version }} - Sécurité par design</p>
    </footer>
  </div>
</template>

<script>
import { ref, onMounted } from "vue";
import DeviceList from "./components/DeviceList.vue";
import FileExplorer from "./components/FileExplorer.vue";

export default {
  name: "App",
  components: {
    DeviceList,
    FileExplorer,
  },
  setup() {
    const version = ref("0.1.0");
    const currentView = ref("devices");
    const selectedPartition = ref(null);

    const handleOpenPartition = (partitionData) => {
      selectedPartition.value = partitionData;
      currentView.value = "explorer";
    };

    const handleCloseExplorer = () => {
      selectedPartition.value = null;
      currentView.value = "devices";
    };

    onMounted(() => {
      // Initialize app
    });

    return {
      version,
      currentView,
      selectedPartition,
      handleOpenPartition,
      handleCloseExplorer,
    };
  },
};
</script>

<style scoped>
.app-header {
  background: #1a1a1a;
  color: #ffffff;
  padding: 1rem 2rem;
  text-align: center;
  border-bottom: 2px solid #333;
}

.app-header h1 {
  margin: 0;
  font-size: 2rem;
  font-weight: bold;
}

.app-header p {
  margin: 0.5rem 0 0 0;
  color: #888;
  font-size: 0.9rem;
}

.app-nav {
  margin-top: 1rem;
  display: flex;
  gap: 0.5rem;
  justify-content: center;
}

.nav-button {
  background: #333;
  color: #fff;
  border: 1px solid #555;
  padding: 0.75rem 1.5rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s;
}

.nav-button:hover {
  background: #444;
}

.nav-button.active {
  background: #00ff88;
  color: #000;
  border-color: #00ff88;
}

.app-main {
  min-height: calc(100vh - 120px);
  padding: 2rem;
  background: #0d1117;
}

.app-footer {
  background: #1a1a1a;
  color: #888;
  padding: 1rem 2rem;
  text-align: center;
  border-top: 1px solid #333;
  font-size: 0.8rem;
}
</style>
