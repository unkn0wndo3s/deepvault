<template>
  <div class="device-list">
    <div class="screen-header">
      <h2>DeepVault — Détecteur de périphériques USB</h2>
    </div>

    <div class="device-table-container">
      <table class="device-table">
        <thead>
          <tr>
            <th>Nom</th>
            <th>Device</th>
            <th>Taille</th>
            <th>Partitions</th>
            <th>Monté</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="device in devices"
            :key="device.device_path"
            class="device-row"
          >
            <td>{{ device.name }}</td>
            <td>{{ device.device_path }}</td>
            <td>{{ formatBytes(device.size) }}</td>
            <td>{{ device.partitions.length }}</td>
            <td>
              <span :class="device.is_mounted ? 'mounted' : 'unmounted'">
                {{ device.is_mounted ? "Oui" : "Non" }}
              </span>
            </td>
            <td>
              <button
                @click="selectDevice(device)"
                class="select-btn"
                :disabled="loading"
              >
                Sélectionner
              </button>
            </td>
          </tr>
        </tbody>
      </table>

      <div v-if="devices.length === 0 && !loading" class="no-devices">
        <p>Aucun périphérique USB détecté</p>
      </div>
    </div>

    <div class="actions">
      <button @click="refreshDevices" class="refresh-btn" :disabled="loading">
        {{ loading ? "Rafraîchissement..." : "Rafraîchir" }}
      </button>
      <button @click="listDisks" class="debug-btn" :disabled="loading">
        🔍 Lister disques
      </button>
      <button @click="showHelp" class="help-btn">Aide rapide</button>
    </div>

    <div v-if="selectedDevice" class="selected-device">
      <h3>Périphérique sélectionné : {{ selectedDevice.name }}</h3>
      <div class="device-info">
        <p><strong>Chemin :</strong> {{ selectedDevice.device_path }}</p>
        <p><strong>Taille :</strong> {{ formatBytes(selectedDevice.size) }}</p>
        <p>
          <strong>Partitions existantes :</strong>
          {{ selectedDevice.partitions.length }}
        </p>
      </div>

      <div class="device-actions">
        <button
          @click="partitionDevice"
          class="partition-btn"
          :disabled="partitioning"
        >
          {{ partitioning ? "⏳ Partitionnement..." : "🔧 Partitionner" }}
        </button>
        <button
          @click="openPublicPartition"
          class="open-btn"
          :disabled="!hasPublicPartition || partitioning"
        >
          📁 Ouvrir partition publique
        </button>
        <button
          @click="openEncryptedPartition"
          class="open-btn"
          :disabled="partitioning"
        >
          🔐 Accéder à la partition chiffrée
        </button>
      </div>

      <!-- Interface de configuration du partitionnement -->
      <div v-if="showPartitionConfig" class="partition-config">
        <h3>Configuration du partitionnement</h3>
        <div class="config-form">
          <div class="form-group">
            <label for="publicSize">Taille partition publique (GB) :</label>
            <input
              id="publicSize"
              v-model.number="partitionConfig.publicSizeGB"
              type="number"
              min="1"
              :max="Math.floor(totalSizeGB - 1)"
              class="form-input"
            />
            <span class="form-help"
              >Maximum: {{ Math.floor(totalSizeGB - 1) }} GB</span
            >
            <span v-if="filesystemWarning" class="form-warning">
              {{ filesystemWarning }}
            </span>
          </div>

          <div class="form-group">
            <label for="publicLabel">Nom de la partition publique :</label>
            <input
              id="publicLabel"
              v-model="partitionConfig.publicLabel"
              type="text"
              placeholder="DEEPVAULT_PUBLIC"
              class="form-input"
            />
          </div>

          <div class="form-group">
            <label for="password"
              >Mot de passe pour la partition chiffrée :</label
            >
            <input
              id="password"
              v-model="partitionConfig.password"
              type="password"
              placeholder="Mot de passe fort"
              class="form-input"
            />
          </div>

          <div class="form-group">
            <label for="confirmPassword">Confirmer le mot de passe :</label>
            <input
              id="confirmPassword"
              v-model="partitionConfig.confirmPassword"
              type="password"
              placeholder="Répéter le mot de passe"
              class="form-input"
            />
          </div>

          <div class="config-actions">
            <button
              @click="confirmPartition"
              class="confirm-btn"
              :disabled="!isConfigValid"
            >
              🔧 Confirmer le partitionnement
            </button>
            <button @click="cancelPartition" class="cancel-btn">
              ❌ Annuler
            </button>
          </div>
        </div>
      </div>

      <div v-if="partitioning" class="partitioning-status">
        <p>⏳ Partitionnement en cours... Veuillez patienter.</p>
        <p class="warning">
          ⚠️ Ne débranchez pas la clé USB pendant cette opération !
        </p>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, onMounted, computed } from "vue";
import { useRouter } from "vue-router";
import { invoke } from "@tauri-apps/api/tauri";

export default {
  name: "DeviceList",
  emits: ["open-partition"],
  setup(props, { emit }) {
    const router = useRouter();
    const devices = ref([]);
    const selectedDevice = ref(null);
    const loading = ref(false);
    const partitioning = ref(false);
    const showPartitionConfig = ref(false);

    const partitionConfig = ref({
      publicSizeGB: 59,
      publicLabel: "DEEPVAULT_PUBLIC",
      password: "",
      confirmPassword: "",
    });

    const formatBytes = (bytes) => {
      if (bytes === 0) return "0 B";
      const k = 1024;
      const sizes = ["B", "KB", "MB", "GB", "TB"];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
    };

    const loadDevices = async () => {
      loading.value = true;
      try {
        devices.value = await invoke("get_usb_devices");
      } catch (error) {
        console.error("Erreur lors du chargement des périphériques:", error);
      } finally {
        loading.value = false;
      }
    };

    const refreshDevices = () => {
      loadDevices();
    };

    const selectDevice = (device) => {
      selectedDevice.value = device;
    };

    // Computed properties pour vérifier les partitions
    const hasPublicPartition = computed(() => {
      if (!selectedDevice.value) return false;
      return selectedDevice.value.partitions.some((p) => p.type === "public");
    });

    const hasEncryptedPartition = computed(() => {
      if (!selectedDevice.value) return false;
      return selectedDevice.value.partitions.some(
        (p) => p.type === "encrypted"
      );
    });

    const totalSizeGB = computed(() => {
      if (!selectedDevice.value) return 0;
      return Math.floor(selectedDevice.value.size / (1024 * 1024 * 1024));
    });

    const isConfigValid = computed(() => {
      return (
        partitionConfig.value.publicSizeGB >= 1 &&
        partitionConfig.value.publicSizeGB <=
          Math.floor(totalSizeGB.value - 1) &&
        partitionConfig.value.publicLabel.trim().length > 0 &&
        partitionConfig.value.password.length >= 8 &&
        partitionConfig.value.password === partitionConfig.value.confirmPassword
      );
    });

    const filesystemWarning = computed(() => {
      if (partitionConfig.value.publicSizeGB > 32) {
        return "⚠️ La partition publique dépassera 32GB et utilisera NTFS au lieu de FAT32";
      }
      return "";
    });

    const partitionDevice = async () => {
      console.log("=== DÉBUT DE LA FONCTION PARTITION DEVICE ===");

      if (!selectedDevice.value) {
        console.log("ERREUR: Aucun périphérique sélectionné");
        return;
      }

      console.log("Périphérique sélectionné:", selectedDevice.value);

      // Afficher l'interface de configuration
      showPartitionConfig.value = true;
    };

    const confirmPartition = async () => {
      console.log("=== CONFIRMATION DU PARTITIONNEMENT ===");
      console.log("Configuration:", partitionConfig.value);

      partitioning.value = true;
      showPartitionConfig.value = false;

      try {
        console.log("Début du partitionnement...");
        console.log("Périphérique sélectionné:", selectedDevice.value);
        console.log(
          "Chemin du périphérique:",
          selectedDevice.value.device_path
        );

        // Appeler la fonction de partitionnement
        console.log("Appel de la fonction partition_device...");

        // Calculer les tailles de partition
        const publicSizeBytes =
          partitionConfig.value.publicSizeGB * 1024 * 1024 * 1024;
        const totalSizeBytes = selectedDevice.value.size;
        const encryptedSizeBytes = Math.max(
          0,
          totalSizeBytes - publicSizeBytes
        );

        console.log("Taille totale:", totalSizeBytes);
        console.log("Taille publique:", publicSizeBytes);
        console.log("Taille chiffrée:", encryptedSizeBytes);

        const result = await invoke("partition_device", {
          devicePath: selectedDevice.value.device_path,
          publicSize: publicSizeBytes,
          encryptedSize: encryptedSizeBytes,
          publicLabel: partitionConfig.value.publicLabel,
          password: partitionConfig.value.password,
        });

        console.log("Résultat du partitionnement:", result);
        window.alert(`Partitionnement terminé : ${result}`);

        // Rafraîchir la liste des périphériques
        console.log("Rafraîchissement de la liste des périphériques...");
        await loadDevices();

        // Resélectionner le périphérique
        const device = devices.value.find(
          (d) => d.device_path === selectedDevice.value.device_path
        );
        if (device) {
          selectedDevice.value = device;
        }
        console.log("Partitionnement terminé avec succès");
      } catch (error) {
        console.error("ERREUR lors du partitionnement:", error);
        console.error("Type d'erreur:", typeof error);
        console.error("Message d'erreur:", error.message);
        console.error("Stack trace:", error.stack);
        window.alert(`Erreur lors du partitionnement : ${error}`);
      } finally {
        console.log("Mise à false de la variable partitioning");
        partitioning.value = false;
        console.log("=== FIN DE LA FONCTION PARTITION DEVICE ===");
      }
    };

    const cancelPartition = () => {
      console.log("Annulation du partitionnement");
      showPartitionConfig.value = false;
      partitioning.value = false;
    };

    const openPublicPartition = () => {
      if (!selectedDevice.value) return;

      const publicPartition = selectedDevice.value.partitions.find(
        (p) => p.type === "public"
      );
      if (publicPartition) {
        emit("open-partition", {
          device: selectedDevice.value,
          partition: publicPartition,
          type: "public",
        });
      }
    };

    const openEncryptedPartition = async () => {
      if (!selectedDevice.value) return;

      console.log("=== ACCÈS À LA PARTITION CHIFFRÉE ===");

      // Demander le mot de passe
      const password = window.prompt(
        "Entrez le mot de passe pour la partition chiffrée:"
      );
      if (!password) {
        console.log("Mot de passe annulé");
        return;
      }

      try {
        console.log("Tentative de montage de la partition chiffrée...");
        const result = await invoke("mount_encrypted_partition", {
          password: password,
        });

        console.log("Chemin de montage:", result);
        window.alert(`Partition chiffrée montée avec succès sur ${result} !`);

        // Ouvrir l'explorateur Windows directement sur la partition montée
        try {
          // Utiliser l'API Windows pour ouvrir l'explorateur
          await invoke("open_explorer", { path: result });
          console.log("Explorateur ouvert sur:", result);
        } catch (explorerError) {
          console.warn(
            "Impossible d'ouvrir l'explorateur automatiquement:",
            explorerError
          );
          // Fallback: essayer d'ouvrir via l'URL
          window.open(`file:///${result}`, "_blank");
        }

        // Émettre l'événement pour l'interface
        emit("open-partition", {
          device: selectedDevice.value,
          partition: { name: "Partition chiffrée", path: result },
          type: "encrypted",
        });
      } catch (error) {
        console.error("Erreur lors du montage:", error);
        window.alert(
          `Erreur lors du montage de la partition chiffrée : ${error}`
        );
      }
    };

    const listDisks = async () => {
      try {
        const disks = await invoke("list_disks");
        alert(`Liste des disques:\n\n${disks}`);
      } catch (error) {
        alert(`Erreur lors de la liste des disques: ${error}`);
      }
    };

    const showHelp = () => {
      alert(
        "Aide rapide :\n\n" +
          "1. Sélectionnez une clé USB\n" +
          "2. Cliquez sur 'Partitionner' pour créer les partitions\n" +
          "3. Utilisez 'Ouvrir partition publique' pour accéder aux fichiers normaux\n" +
          "4. Utilisez 'Ouvrir partition chiffrée' pour accéder aux fichiers sécurisés"
      );
    };

    onMounted(() => {
      loadDevices();
    });

    return {
      devices,
      selectedDevice,
      loading,
      partitioning,
      showPartitionConfig,
      partitionConfig,
      totalSizeGB,
      isConfigValid,
      filesystemWarning,
      hasPublicPartition,
      hasEncryptedPartition,
      formatBytes,
      refreshDevices,
      selectDevice,
      partitionDevice,
      confirmPartition,
      cancelPartition,
      openPublicPartition,
      openEncryptedPartition,
      listDisks,
      showHelp,
    };
  },
};
</script>

<style scoped>
.device-list {
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

.device-table-container {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 6px;
  overflow: hidden;
  margin-bottom: 2rem;
}

.device-table {
  width: 100%;
  border-collapse: collapse;
}

.device-table th {
  background: #21262d;
  color: #f0f6fc;
  padding: 1rem;
  text-align: left;
  font-weight: 600;
  border-bottom: 1px solid #30363d;
}

.device-table td {
  padding: 1rem;
  color: #c9d1d9;
  border-bottom: 1px solid #30363d;
}

.device-row:hover {
  background: #21262d;
}

.mounted {
  color: #3fb950;
  font-weight: bold;
}

.unmounted {
  color: #f85149;
  font-weight: bold;
}

.select-btn {
  background: #238636;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: 500;
}

.select-btn:hover:not(:disabled) {
  background: #2ea043;
}

.select-btn:disabled {
  background: #484f58;
  cursor: not-allowed;
}

.actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  margin-bottom: 2rem;
}

.refresh-btn,
.debug-btn,
.help-btn {
  padding: 0.75rem 1.5rem;
  border: 1px solid #30363d;
  border-radius: 6px;
  background: #21262d;
  color: #c9d1d9;
  cursor: pointer;
  font-weight: 500;
}

.refresh-btn:hover:not(:disabled) {
  background: #30363d;
}

.debug-btn:hover:not(:disabled) {
  background: #30363d;
}

.help-btn:hover {
  background: #30363d;
}

.refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.selected-device {
  background: #0d1117;
  border: 2px solid #238636;
  border-radius: 6px;
  padding: 1.5rem;
  margin-top: 1rem;
}

.selected-device h3 {
  color: #c9d1d9;
  margin: 0 0 1rem 0;
  font-size: 1.2rem;
  text-align: center;
}

.device-info {
  margin-bottom: 1.5rem;
}

.device-info p {
  color: #c9d1d9;
  margin: 0.5rem 0;
  font-size: 0.9rem;
}

.device-actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  flex-wrap: wrap;
}

.partition-btn,
.open-btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  font-size: 0.9rem;
  transition: all 0.2s;
}

.partition-btn {
  background: #f85149;
  color: white;
}

.partition-btn:hover:not(:disabled) {
  background: #da3633;
}

.open-btn {
  background: #238636;
  color: white;
}

.open-btn:hover:not(:disabled) {
  background: #2ea043;
}

.partition-btn:disabled,
.open-btn:disabled {
  background: #484f58;
  cursor: not-allowed;
  opacity: 0.6;
}

.no-devices {
  text-align: center;
  padding: 3rem;
  color: #7d8590;
}

.no-devices p {
  font-size: 1.1rem;
  margin: 0;
}

.partitioning-status {
  background: #0d1117;
  border: 2px solid #f85149;
  border-radius: 6px;
  padding: 1rem;
  margin-top: 1rem;
  text-align: center;
}

.partitioning-status p {
  color: #c9d1d9;
  margin: 0.5rem 0;
  font-size: 0.9rem;
}

.partitioning-status .warning {
  color: #f85149;
  font-weight: bold;
}

/* Interface de configuration du partitionnement */
.partition-config {
  background: #0d1117;
  border: 2px solid #30363d;
  border-radius: 8px;
  padding: 1.5rem;
  margin-top: 1rem;
}

.partition-config h3 {
  color: #c9d1d9;
  margin-bottom: 1rem;
  text-align: center;
}

.config-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form-group label {
  color: #c9d1d9;
  font-weight: 500;
}

.form-input {
  background: #21262d;
  border: 1px solid #30363d;
  border-radius: 6px;
  color: #c9d1d9;
  padding: 0.75rem;
  font-size: 0.9rem;
}

.form-input:focus {
  outline: none;
  border-color: #58a6ff;
  box-shadow: 0 0 0 2px rgba(88, 166, 255, 0.1);
}

.form-help {
  color: #8b949e;
  font-size: 0.8rem;
}

.form-warning {
  color: #f85149;
  font-size: 0.8rem;
  font-weight: 500;
}

.config-actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
  margin-top: 1rem;
}

.confirm-btn {
  background: #238636;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 0.75rem 1.5rem;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.confirm-btn:hover:not(:disabled) {
  background: #2ea043;
}

.confirm-btn:disabled {
  background: #484f58;
  cursor: not-allowed;
}

.cancel-btn {
  background: #da3633;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 0.75rem 1.5rem;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.cancel-btn:hover {
  background: #f85149;
}
</style>
