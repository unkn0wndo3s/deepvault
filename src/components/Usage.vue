<template>
  <div class="usage-screen">
    <div class="screen-header">
      <h2>Utilisation - Montage/Démontage/Effacement</h2>
    </div>

    <div class="volumes-section">
      <h3>Volumes disponibles sur le périphérique</h3>
      <div class="volumes-list">
        <div
          v-for="volume in volumes"
          :key="volume.name"
          class="volume-card"
          :class="{ mounted: volume.isMounted }"
        >
          <div class="volume-info">
            <h4>{{ volume.name }}</h4>
            <p class="volume-type">{{ volume.type }}</p>
            <p class="volume-status">
              <span :class="volume.isMounted ? 'mounted' : 'unmounted'">
                {{ volume.isMounted ? "Monté" : "Non monté" }}
              </span>
              <span v-if="volume.mountPoint" class="mount-point">
                sur {{ volume.mountPoint }}
              </span>
            </p>
          </div>

          <div class="volume-actions">
            <button
              v-if="!volume.isMounted"
              @click="mountVolume(volume)"
              class="action-btn mount-btn"
              :disabled="mounting"
            >
              {{ mounting === volume.name ? "Montage..." : "Monter" }}
            </button>

            <button
              v-if="volume.isMounted"
              @click="unmountVolume(volume)"
              class="action-btn unmount-btn"
              :disabled="unmounting"
            >
              {{ unmounting === volume.name ? "Démontage..." : "Démonter" }}
            </button>

            <button
              @click="wipeVolume(volume)"
              class="action-btn wipe-btn"
              :disabled="wiping"
            >
              {{ wiping === volume.name ? "Effacement..." : "Effacer" }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="advanced-section">
      <h3>Options avancées</h3>
      <div class="advanced-options">
        <div class="option-group">
          <label class="checkbox-label">
            <input
              type="checkbox"
              v-model="wipeHeaderOnly"
              class="checkbox-input"
            />
            <span class="checkbox-text">Écraser header seulement</span>
          </label>
        </div>

        <div class="option-group">
          <label class="checkbox-label">
            <input
              type="checkbox"
              v-model="wipeEverything"
              class="checkbox-input"
            />
            <span class="checkbox-text">Écraser tout (overwrite)</span>
          </label>
        </div>
      </div>
    </div>

    <div v-if="showWipeWarning" class="wipe-warning">
      <div class="warning-icon">⚠</div>
      <div class="warning-content">
        <h4>ATTENTION - EFFACEMENT IRRÉVERSIBLE</h4>
        <p>
          Cette action va effacer définitivement toutes les données du volume
          sélectionné.
        </p>
        <p>Cette opération ne peut pas être annulée.</p>
        <div class="warning-actions">
          <button @click="confirmWipe" class="confirm-wipe-btn">
            Confirmer l'effacement
          </button>
          <button @click="cancelWipe" class="cancel-wipe-btn">Annuler</button>
        </div>
      </div>
    </div>

    <div class="actions">
      <button @click="goToLogs" class="logs-btn">Voir les logs</button>
      <button @click="goToDeviceList" class="device-list-btn">
        Nouveau périphérique
      </button>
    </div>
  </div>
</template>

<script>
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/tauri";

export default {
  name: "Usage",
  setup() {
    const router = useRouter();
    const volumes = ref([
      {
        name: "public",
        type: "Partition publique",
        isMounted: false,
        mountPoint: null,
      },
      {
        name: "encrypted",
        type: "Volume chiffré",
        isMounted: false,
        mountPoint: null,
      },
      {
        name: "hidden",
        type: "Volume caché",
        isMounted: false,
        mountPoint: null,
      },
    ]);
    const mounting = ref(null);
    const unmounting = ref(null);
    const wiping = ref(null);
    const wipeHeaderOnly = ref(false);
    const wipeEverything = ref(false);
    const showWipeWarning = ref(false);
    const volumeToWipe = ref(null);

    const mountVolume = async (volume) => {
      mounting.value = volume.name;

      try {
        const password = prompt("Entrez le mot de passe:");
        if (!password) {
          mounting.value = null;
          return;
        }

        const result = await invoke("mount_volume", {
          devicePath: "/dev/sdb", // TODO: Get from context
          volumeName: volume.name,
          password: password,
        });

        console.log("Mount result:", result);

        // Update volume status
        volume.isMounted = true;
        volume.mountPoint = "/mnt/" + volume.name;
      } catch (error) {
        console.error("Error mounting volume:", error);
        alert("Erreur lors du montage: " + error);
      } finally {
        mounting.value = null;
      }
    };

    const unmountVolume = async (volume) => {
      unmounting.value = volume.name;

      try {
        const result = await invoke("unmount_volume", {
          volumeName: volume.name,
        });

        console.log("Unmount result:", result);

        // Update volume status
        volume.isMounted = false;
        volume.mountPoint = null;
      } catch (error) {
        console.error("Error unmounting volume:", error);
        alert("Erreur lors du démontage: " + error);
      } finally {
        unmounting.value = null;
      }
    };

    const wipeVolume = (volume) => {
      volumeToWipe.value = volume;
      showWipeWarning.value = true;
    };

    const confirmWipe = async () => {
      if (!volumeToWipe.value) return;

      wiping.value = volumeToWipe.value.name;
      showWipeWarning.value = false;

      try {
        const result = await invoke("wipe_device", {
          devicePath: "/dev/sdb", // TODO: Get from context
          headerOnly: wipeHeaderOnly.value,
        });

        console.log("Wipe result:", result);
        alert("Effacement terminé");
      } catch (error) {
        console.error("Error wiping device:", error);
        alert("Erreur lors de l'effacement: " + error);
      } finally {
        wiping.value = null;
        volumeToWipe.value = null;
      }
    };

    const cancelWipe = () => {
      showWipeWarning.value = false;
      volumeToWipe.value = null;
    };

    const goToLogs = () => {
      router.push("/logs");
    };

    const goToDeviceList = () => {
      router.push("/");
    };

    onMounted(() => {
      // Load volumes status
      // TODO: Implement volume status loading
    });

    return {
      volumes,
      mounting,
      unmounting,
      wiping,
      wipeHeaderOnly,
      wipeEverything,
      showWipeWarning,
      volumeToWipe,
      mountVolume,
      unmountVolume,
      wipeVolume,
      confirmWipe,
      cancelWipe,
      goToLogs,
      goToDeviceList,
    };
  },
};
</script>

<style scoped>
.usage-screen {
  max-width: 900px;
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

.volumes-section {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 2rem;
  margin-bottom: 2rem;
}

.volumes-section h3 {
  color: #f0f6fc;
  margin: 0 0 1.5rem 0;
  font-size: 1.2rem;
}

.volumes-list {
  display: grid;
  gap: 1rem;
}

.volume-card {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 1.5rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  transition: all 0.2s ease;
}

.volume-card.mounted {
  border-color: #3fb950;
  background: #0d1117;
}

.volume-info h4 {
  color: #f0f6fc;
  margin: 0 0 0.5rem 0;
  font-size: 1.1rem;
}

.volume-type {
  color: #7d8590;
  margin: 0 0 0.5rem 0;
  font-size: 0.9rem;
}

.volume-status {
  margin: 0;
  font-size: 0.9rem;
}

.volume-status .mounted {
  color: #3fb950;
  font-weight: bold;
}

.volume-status .unmounted {
  color: #f85149;
  font-weight: bold;
}

.mount-point {
  color: #58a6ff;
  margin-left: 0.5rem;
}

.volume-actions {
  display: flex;
  gap: 0.5rem;
}

.action-btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
  font-size: 0.9rem;
  transition: all 0.2s ease;
}

.mount-btn {
  background: #238636;
  color: white;
}

.mount-btn:hover:not(:disabled) {
  background: #2ea043;
}

.unmount-btn {
  background: #f0a020;
  color: white;
}

.unmount-btn:hover:not(:disabled) {
  background: #d48806;
}

.wipe-btn {
  background: #f85149;
  color: white;
}

.wipe-btn:hover:not(:disabled) {
  background: #da3633;
}

.action-btn:disabled {
  background: #484f58;
  cursor: not-allowed;
  opacity: 0.5;
}

.advanced-section {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 2rem;
  margin-bottom: 2rem;
}

.advanced-section h3 {
  color: #f0f6fc;
  margin: 0 0 1.5rem 0;
  font-size: 1.2rem;
}

.advanced-options {
  display: grid;
  gap: 1rem;
}

.option-group {
  margin-bottom: 1rem;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  cursor: pointer;
}

.checkbox-input {
  width: 18px;
  height: 18px;
  accent-color: #238636;
}

.checkbox-text {
  color: #c9d1d9;
  font-size: 0.9rem;
}

.wipe-warning {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: #1a1a1a;
  border: 2px solid #f85149;
  border-radius: 8px;
  padding: 2rem;
  max-width: 500px;
  z-index: 1000;
}

.warning-icon {
  color: #f85149;
  font-size: 2rem;
  text-align: center;
  margin-bottom: 1rem;
}

.warning-content h4 {
  color: #f85149;
  margin: 0 0 1rem 0;
  text-align: center;
  font-size: 1.2rem;
}

.warning-content p {
  color: #c9d1d9;
  margin: 0.5rem 0;
  text-align: center;
}

.warning-actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  margin-top: 1.5rem;
}

.confirm-wipe-btn,
.cancel-wipe-btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  font-size: 1rem;
}

.confirm-wipe-btn {
  background: #f85149;
  color: white;
}

.confirm-wipe-btn:hover {
  background: #da3633;
}

.cancel-wipe-btn {
  background: #484f58;
  color: #c9d1d9;
}

.cancel-wipe-btn:hover {
  background: #6e7681;
}

.actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
}

.logs-btn,
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

.logs-btn:hover,
.device-list-btn:hover {
  background: #30363d;
  border-color: #58a6ff;
}
</style>
