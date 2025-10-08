<template>
  <div class="logs-screen">
    <div class="screen-header">
      <h2>Logs & Aide</h2>
    </div>

    <div class="logs-section">
      <div class="logs-header">
        <h3>Journal des opérations</h3>
        <div class="logs-actions">
          <button @click="clearLogs" class="clear-btn">Effacer</button>
          <button @click="exportLogs" class="export-btn">Exporter</button>
        </div>
      </div>

      <div class="logs-content">
        <div
          v-for="(log, index) in logs"
          :key="index"
          class="log-entry"
          :class="log.level"
        >
          <span class="log-timestamp">{{ log.timestamp }}</span>
          <span class="log-level">{{ log.level.toUpperCase() }}</span>
          <span class="log-message">{{ log.message }}</span>
        </div>

        <div v-if="logs.length === 0" class="no-logs">
          <p>Aucun log disponible</p>
        </div>
      </div>
    </div>

    <div class="help-section">
      <h3>Recommandations de sécurité</h3>
      <div class="help-content">
        <div class="help-item">
          <h4>🔒 Mot de passe fort</h4>
          <p>
            Utilisez un mot de passe d'au moins 12 caractères avec des
            majuscules, minuscules, chiffres et symboles.
          </p>
        </div>

        <div class="help-item">
          <h4>💾 Sauvegarde du header</h4>
          <p>
            Le fichier header (.dv_meta) contient des informations cruciales.
            Gardez-en une copie de sauvegarde.
          </p>
        </div>

        <div class="help-item">
          <h4>🔄 Démontage automatique</h4>
          <p>
            Les volumes se démontent automatiquement après 5 minutes
            d'inactivité pour votre sécurité.
          </p>
        </div>

        <div class="help-item">
          <h4>🗑️ Effacement sécurisé</h4>
          <p>
            L'effacement utilise le standard DoD 5220.22-M (3 passes) pour
            garantir la destruction des données.
          </p>
        </div>

        <div class="help-item">
          <h4>🔍 Volume caché</h4>
          <p>
            Le volume caché permet le déni plausible en cas de contrainte
            physique ou légale.
          </p>
        </div>

        <div class="help-item">
          <h4>⚠️ Privilèges administrateur</h4>
          <p>
            L'application nécessite des privilèges élevés pour manipuler les
            partitions. Fermez l'application après utilisation.
          </p>
        </div>
      </div>
    </div>

    <div class="actions">
      <button @click="goBack" class="back-btn">Retour</button>
      <button @click="goToDeviceList" class="device-list-btn">
        Nouveau périphérique
      </button>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";

export default {
  name: "Logs",
  setup() {
    const router = useRouter();
    const logs = ref([]);

    const addLog = (message, level = "info") => {
      const now = new Date();
      const timestamp = now.toLocaleString("fr-FR");
      logs.value.push({
        timestamp,
        level,
        message,
      });
    };

    const clearLogs = () => {
      logs.value = [];
    };

    const exportLogs = () => {
      const logContent = logs.value
        .map(
          (log) =>
            `[${log.timestamp}] ${log.level.toUpperCase()}: ${log.message}`
        )
        .join("\n");

      const blob = new Blob([logContent], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `deepvault-logs-${
        new Date().toISOString().split("T")[0]
      }.txt`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    };

    const goBack = () => {
      router.go(-1);
    };

    const goToDeviceList = () => {
      router.push("/");
    };

    onMounted(() => {
      // Add some sample logs
      addLog("Application démarrée", "info");
      addLog("Périphérique USB détecté: /dev/sdb", "info");
      addLog("Configuration de sécurité validée", "info");
      addLog("Volume chiffré créé avec succès", "success");
      addLog("Montage du volume réussi", "success");
      addLog("Démontage du volume effectué", "info");
    });

    return {
      logs,
      clearLogs,
      exportLogs,
      goBack,
      goToDeviceList,
    };
  },
};
</script>

<style scoped>
.logs-screen {
  max-width: 1000px;
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

.logs-section {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  margin-bottom: 2rem;
  overflow: hidden;
}

.logs-header {
  background: #21262d;
  padding: 1rem 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #30363d;
}

.logs-header h3 {
  color: #f0f6fc;
  margin: 0;
  font-size: 1.2rem;
}

.logs-actions {
  display: flex;
  gap: 0.5rem;
}

.clear-btn,
.export-btn {
  padding: 0.5rem 1rem;
  border: 1px solid #30363d;
  border-radius: 4px;
  background: #21262d;
  color: #c9d1d9;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s ease;
}

.clear-btn:hover,
.export-btn:hover {
  background: #30363d;
  border-color: #58a6ff;
}

.logs-content {
  max-height: 400px;
  overflow-y: auto;
  padding: 1rem 2rem;
}

.log-entry {
  display: grid;
  grid-template-columns: 150px 80px 1fr;
  gap: 1rem;
  padding: 0.5rem 0;
  border-bottom: 1px solid #30363d;
  font-family: "Courier New", monospace;
  font-size: 0.9rem;
}

.log-entry:last-child {
  border-bottom: none;
}

.log-timestamp {
  color: #7d8590;
}

.log-level {
  font-weight: bold;
}

.log-entry.info .log-level {
  color: #58a6ff;
}

.log-entry.success .log-level {
  color: #3fb950;
}

.log-entry.warning .log-level {
  color: #f0a020;
}

.log-entry.error .log-level {
  color: #f85149;
}

.log-message {
  color: #c9d1d9;
}

.no-logs {
  text-align: center;
  padding: 2rem;
  color: #7d8590;
}

.help-section {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 2rem;
  margin-bottom: 2rem;
}

.help-section h3 {
  color: #f0f6fc;
  margin: 0 0 1.5rem 0;
  font-size: 1.2rem;
}

.help-content {
  display: grid;
  gap: 1.5rem;
}

.help-item {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 1.5rem;
}

.help-item h4 {
  color: #f0f6fc;
  margin: 0 0 0.75rem 0;
  font-size: 1.1rem;
}

.help-item p {
  color: #c9d1d9;
  margin: 0;
  line-height: 1.5;
}

.actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
}

.back-btn,
.device-list-btn {
  padding: 0.75rem 2rem;
  border: 1px solid #30363d;
  border-radius: 6px;
  background: #21262d;
  color: #c9d1d9;
  cursor: pointer;
  font-weight: 500;
  font-size: 1rem;
  transition: all 0.2s ease;
}

.back-btn:hover,
.device-list-btn:hover {
  background: #30363d;
  border-color: #58a6ff;
}
</style>
