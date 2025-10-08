<template>
  <div class="progress-screen">
    <div class="screen-header">
      <h2>Création en cours</h2>
    </div>

    <div class="progress-container">
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: progress + '%' }"></div>
      </div>
      <div class="progress-text">{{ progress }}%</div>
    </div>

    <div class="log-container">
      <div class="log-header">
        <h3>Journal de progression</h3>
      </div>
      <div class="log-content">
        <div
          v-for="(log, index) in logs"
          :key="index"
          class="log-entry"
          :class="log.type"
        >
          <span class="log-time">{{ log.time }}</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
    </div>

    <div class="actions">
      <button
        @click="cancelOperation"
        class="cancel-btn"
        :disabled="!canCancel"
      >
        {{ canCancel ? "Annuler" : "Opération en cours..." }}
      </button>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, onUnmounted } from "vue";
import { useRouter } from "vue-router";

export default {
  name: "Progress",
  setup() {
    const router = useRouter();
    const progress = ref(0);
    const logs = ref([]);
    const canCancel = ref(true);
    const intervalId = ref(null);

    const addLog = (message, type = "info") => {
      const now = new Date();
      const time = now.toLocaleTimeString();
      logs.value.push({
        time,
        message,
        type,
      });
    };

    const simulateProgress = () => {
      const steps = [
        { progress: 10, message: "En attente d'élévation...", type: "warning" },
        {
          progress: 25,
          message: "Création de la partition publique...",
          type: "info",
        },
        {
          progress: 40,
          message: "Formatage de la partition publique...",
          type: "info",
        },
        {
          progress: 55,
          message: "Initialisation du conteneur chiffré...",
          type: "info",
        },
        {
          progress: 70,
          message: "Dérivation de la clé de chiffrement...",
          type: "info",
        },
        {
          progress: 85,
          message: "Écriture du header chiffré...",
          type: "info",
        },
        { progress: 95, message: "Finalisation...", type: "info" },
        {
          progress: 100,
          message: "Création terminée avec succès!",
          type: "success",
        },
      ];

      let currentStep = 0;

      const updateProgress = () => {
        if (currentStep < steps.length) {
          const step = steps[currentStep];
          progress.value = step.progress;
          addLog(step.message, step.type);
          currentStep++;
        } else {
          clearInterval(intervalId.value);
          canCancel.value = false;
          // Redirect to usage screen after completion
          setTimeout(() => {
            router.push("/usage");
          }, 2000);
        }
      };

      // Start with initial log
      addLog("Démarrage de la création...", "info");

      // Update progress every 2 seconds
      intervalId.value = setInterval(updateProgress, 2000);
    };

    const cancelOperation = () => {
      if (canCancel.value) {
        clearInterval(intervalId.value);
        addLog("Opération annulée par l'utilisateur", "warning");
        canCancel.value = false;

        // Redirect back to device list after a short delay
        setTimeout(() => {
          router.push("/");
        }, 1500);
      }
    };

    onMounted(() => {
      simulateProgress();
    });

    onUnmounted(() => {
      if (intervalId.value) {
        clearInterval(intervalId.value);
      }
    });

    return {
      progress,
      logs,
      canCancel,
      cancelOperation,
    };
  },
};
</script>

<style scoped>
.progress-screen {
  max-width: 800px;
  margin: 0 auto;
}

.screen-header {
  text-align: center;
  margin-bottom: 2rem;
}

.screen-header h2 {
  color: #ffffff;
  font-size: 1.5rem;
  margin: 0;
}

.progress-container {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 2rem;
  margin-bottom: 2rem;
  text-align: center;
}

.progress-bar {
  width: 100%;
  height: 20px;
  background: #0d1117;
  border-radius: 10px;
  overflow: hidden;
  margin-bottom: 1rem;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #238636, #2ea043);
  border-radius: 10px;
  transition: width 0.5s ease;
}

.progress-text {
  color: #c9d1d9;
  font-size: 1.2rem;
  font-weight: 600;
}

.log-container {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 8px;
  margin-bottom: 2rem;
  overflow: hidden;
}

.log-header {
  background: #21262d;
  padding: 1rem;
  border-bottom: 1px solid #30363d;
}

.log-header h3 {
  color: #f0f6fc;
  margin: 0;
  font-size: 1.1rem;
}

.log-content {
  max-height: 300px;
  overflow-y: auto;
  padding: 1rem;
}

.log-entry {
  display: flex;
  gap: 1rem;
  margin-bottom: 0.5rem;
  font-family: "Courier New", monospace;
  font-size: 0.9rem;
}

.log-time {
  color: #7d8590;
  min-width: 80px;
}

.log-message {
  color: #c9d1d9;
  flex: 1;
}

.log-entry.info .log-message {
  color: #58a6ff;
}

.log-entry.warning .log-message {
  color: #f0a020;
}

.log-entry.success .log-message {
  color: #3fb950;
  font-weight: 600;
}

.log-entry.error .log-message {
  color: #f85149;
}

.actions {
  display: flex;
  justify-content: center;
}

.cancel-btn {
  padding: 0.75rem 2rem;
  background: #f85149;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  font-size: 1rem;
  transition: all 0.2s ease;
}

.cancel-btn:hover:not(:disabled) {
  background: #da3633;
}

.cancel-btn:disabled {
  background: #484f58;
  cursor: not-allowed;
  opacity: 0.5;
}
</style>
