<template>
  <div class="file-explorer">
    <div class="header">
      <div class="header-top">
        <h2>Explorateur de fichiers sécurisé</h2>
        <button @click="closeExplorer" class="close-explorer-btn">
          ← Retour
        </button>
      </div>
      <div class="partition-info" v-if="partitionInfo">
        <p><strong>Périphérique :</strong> {{ partitionInfo.device.name }}</p>
        <p>
          <strong>Partition :</strong> {{ partitionInfo.partition.name }} ({{
            partitionType === "public" ? "Publique" : "Chiffrée"
          }})
        </p>
      </div>
      <div class="path-bar">
        <span class="current-path">{{ currentPath }}</span>
        <button @click="goUp" :disabled="!canGoUp" class="up-button">↑</button>
      </div>
    </div>

    <div class="password-section" v-if="!isAuthenticated">
      <div class="password-form">
        <h3>Accès à la partition chiffrée</h3>
        <input
          v-model="password"
          type="password"
          placeholder="Mot de passe"
          @keyup.enter="authenticate"
          class="password-input"
        />
        <button @click="authenticate" :disabled="!password" class="auth-button">
          Accéder
        </button>
        <p v-if="authError" class="error">{{ authError }}</p>
      </div>
    </div>

    <div v-else class="file-content">
      <div class="toolbar">
        <button @click="refresh" class="refresh-button">🔄 Actualiser</button>
        <button @click="createFolder" class="new-folder-button">
          📁 Nouveau dossier
        </button>
        <button @click="uploadFile" class="upload-button">
          📤 Importer fichier
        </button>
      </div>

      <div class="file-list">
        <div class="file-item header">
          <div class="file-name">Nom</div>
          <div class="file-size">Taille</div>
          <div class="file-modified">Modifié</div>
          <div class="file-actions">Actions</div>
        </div>

        <div
          v-for="file in files"
          :key="file.path"
          class="file-item"
          :class="{ directory: file.is_directory }"
          @dblclick="handleFileClick(file)"
        >
          <div class="file-name">
            <span class="file-icon">
              {{ file.is_directory ? "📁" : getFileIcon(file.name) }}
            </span>
            {{ file.name }}
          </div>
          <div class="file-size">
            {{ file.is_directory ? "-" : formatFileSize(file.size) }}
          </div>
          <div class="file-modified">
            {{ formatDate(file.modified) }}
          </div>
          <div class="file-actions">
            <button @click="handleFileClick(file)" class="action-button">
              {{ file.is_directory ? "Ouvrir" : "Ouvrir" }}
            </button>
            <button @click="deleteFile(file)" class="action-button delete">
              Supprimer
            </button>
          </div>
        </div>

        <div v-if="files.length === 0" class="empty-state">
          <p>Ce dossier est vide</p>
        </div>
      </div>
    </div>

    <!-- Modal pour l'éditeur de fichier -->
    <div v-if="showEditor" class="modal-overlay" @click="closeEditor">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>{{ editingFile?.name }}</h3>
          <button @click="closeEditor" class="close-button">×</button>
        </div>
        <div class="modal-body">
          <textarea
            v-if="editingFile && isTextFile(editingFile.name)"
            v-model="fileContent"
            class="file-editor"
            placeholder="Contenu du fichier..."
          ></textarea>
          <div v-else class="binary-file">
            <p>Fichier binaire détecté</p>
            <p>Taille: {{ formatFileSize(editingFile?.size || 0) }}</p>
            <button @click="downloadFile" class="download-button">
              Télécharger
            </button>
          </div>
        </div>
        <div class="modal-footer">
          <button @click="saveFile" class="save-button">Sauvegarder</button>
          <button @click="closeEditor" class="cancel-button">Annuler</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { invoke } from "@tauri-apps/api/tauri";

export default {
  name: "FileExplorer",
  props: {
    partitionInfo: {
      type: Object,
      default: null,
    },
  },
  emits: ["close-explorer"],
  data() {
    return {
      isAuthenticated: false,
      password: "",
      authError: "",
      currentPath: "/",
      files: [],
      showEditor: false,
      editingFile: null,
      fileContent: "",
      mountedPath: "",
      partitionType: "public", // "public" ou "encrypted"
    };
  },
  computed: {
    canGoUp() {
      return this.currentPath !== "/" && this.currentPath !== this.mountedPath;
    },
  },
  async mounted() {
    if (this.partitionInfo) {
      this.partitionType = this.partitionInfo.type;

      if (this.partitionType === "public") {
        // Pour la partition publique, pas besoin d'authentification
        this.mountedPath =
          this.partitionInfo.partition.mount_point ||
          `/mnt/${this.partitionInfo.partition.name}`;
        this.isAuthenticated = true;
        this.currentPath = this.mountedPath;
        await this.loadFiles(this.currentPath);
      } else if (this.partitionInfo.partition.sessionId) {
        // Pour la partition chiffrée avec accès direct, utiliser la session
        this.mountedPath = `/encrypted/${this.partitionInfo.partition.sessionId}`;
        this.isAuthenticated = true;
        this.currentPath = this.mountedPath;
        await this.loadEncryptedFiles(
          this.partitionInfo.partition.sessionId,
          "/"
        );
      } else {
        // Pour la partition chiffrée sans session, demander l'authentification
        this.mountedPath = "/encrypted";
        await this.checkMountStatus();
      }
    }
  },
  methods: {
    async authenticate() {
      try {
        if (this.password.length < 8) {
          this.authError =
            "Le mot de passe doit contenir au moins 8 caractères";
          return;
        }

        if (!this.partitionInfo) {
          this.authError = "Aucune partition sélectionnée";
          return;
        }

        // Accéder directement à la partition chiffrée avec le mot de passe
        const sessionId = await invoke("access_encrypted_partition", {
          password: this.password,
        });

        if (sessionId) {
          this.isAuthenticated = true;
          this.authError = "";
          this.mountedPath = `/encrypted/${sessionId}`;
          this.currentPath = this.mountedPath;
          await this.loadEncryptedFiles(sessionId, "/");
        } else {
          this.authError = "Échec de l'accès à la partition";
        }
      } catch (error) {
        this.authError = "Erreur d'authentification: " + error;
      }
    },

    async checkMountStatus() {
      try {
        const status = await invoke("get_mount_status", {
          volumeName: "secure_vault",
        });
        if (status === "Monté") {
          this.isAuthenticated = true;
          this.mountedPath = "/mnt/secure_vault";
          this.currentPath = this.mountedPath;
          await this.loadFiles(this.currentPath);
        }
      } catch (error) {
        console.error("Erreur lors de la vérification du statut:", error);
      }
    },

    async loadFiles(path) {
      try {
        this.files = await invoke("list_files", { path });
      } catch (error) {
        console.error("Erreur lors du chargement des fichiers:", error);
        this.files = [];
      }
    },

    async loadEncryptedFiles(sessionId, path) {
      try {
        console.log(
          `Chargement des fichiers chiffrés pour la session ${sessionId} dans ${path}`
        );
        this.files = await invoke("list_encrypted_files", {
          sessionId: sessionId,
          path: path,
        });
        console.log(`✅ ${this.files.length} fichiers chiffrés chargés`);
      } catch (error) {
        console.error(
          "Erreur lors du chargement des fichiers chiffrés:",
          error
        );
        this.files = [];
      }
    },

    async refresh() {
      if (this.partitionType === "public") {
        await this.loadFiles(this.currentPath);
      } else {
        // Pour les partitions chiffrées, extraire l'ID de session du chemin
        const sessionId = this.extractSessionId(this.currentPath);
        if (sessionId) {
          await this.loadEncryptedFiles(sessionId, this.currentPath);
        }
      }
    },

    async handleFileClick(file) {
      if (file.is_directory) {
        this.currentPath = file.path;
        if (this.partitionType === "public") {
          await this.loadFiles(this.currentPath);
        } else {
          const sessionId = this.extractSessionId(this.currentPath);
          if (sessionId) {
            await this.loadEncryptedFiles(sessionId, this.currentPath);
          }
        }
      } else {
        await this.openFile(file);
      }
    },

    extractSessionId(path) {
      // Extraire l'ID de session du chemin /encrypted/sessionId/...
      const match = path.match(/\/encrypted\/([^\/]+)/);
      return match ? match[1] : null;
    },

    async openFile(file) {
      try {
        if (this.isTextFile(file.name)) {
          let content;
          if (this.partitionType === "encrypted") {
            // Pour les fichiers chiffrés
            const sessionId = this.extractSessionId(this.currentPath);
            if (sessionId) {
              content = await invoke("read_encrypted_file", {
                sessionId: sessionId,
                filePath: file.path,
              });
            } else {
              throw new Error("Session chiffrée non trouvée");
            }
          } else {
            // Pour les fichiers normaux
            const contentBytes = await invoke("read_file", {
              filePath: file.path,
            });
            content = new TextDecoder("utf-8").decode(contentBytes);
          }
          this.fileContent = content;
          this.editingFile = file;
          this.showEditor = true;
        } else {
          // Pour les fichiers binaires, on peut les télécharger
          this.editingFile = file;
          this.showEditor = true;
        }
      } catch (error) {
        console.error("Erreur lors de l'ouverture du fichier:", error);
        alert(`Erreur lors de l'ouverture du fichier: ${error}`);
      }
    },

    async saveFile() {
      if (!this.editingFile) return;

      try {
        if (this.partitionType === "encrypted") {
          // Pour les fichiers chiffrés
          const sessionId = this.extractSessionId(this.currentPath);
          if (sessionId) {
            await invoke("write_encrypted_file", {
              sessionId: sessionId,
              filePath: this.editingFile.path,
              content: this.fileContent,
            });
          } else {
            throw new Error("Session chiffrée non trouvée");
          }
        } else {
          // Pour les fichiers normaux
          const content = new TextEncoder("utf-8").encode(this.fileContent);
          await invoke("write_file", {
            filePath: this.editingFile.path,
            content: Array.from(content),
          });
        }
        this.closeEditor();
        await this.refresh();
      } catch (error) {
        console.error("Erreur lors de la sauvegarde:", error);
        alert(`Erreur lors de la sauvegarde: ${error}`);
      }
    },

    async deleteFile(file) {
      if (!confirm(`Êtes-vous sûr de vouloir supprimer "${file.name}" ?`)) {
        return;
      }

      try {
        if (this.partitionType === "encrypted") {
          // Pour les fichiers chiffrés
          const sessionId = this.extractSessionId(this.currentPath);
          if (sessionId) {
            await invoke("delete_encrypted_file", {
              sessionId: sessionId,
              filePath: file.path,
            });
          } else {
            throw new Error("Session chiffrée non trouvée");
          }
        } else {
          // Pour les fichiers normaux
          await invoke("delete_file", { filePath: file.path });
        }
        await this.refresh();
      } catch (error) {
        console.error("Erreur lors de la suppression:", error);
        alert(`Erreur lors de la suppression: ${error}`);
      }
    },

    async createFolder() {
      const name = prompt("Nom du nouveau dossier:");
      if (!name) return;

      try {
        const newPath = this.currentPath.endsWith("/")
          ? `${this.currentPath}${name}`
          : `${this.currentPath}/${name}`;

        if (this.partitionType === "encrypted") {
          // Pour les dossiers chiffrés
          const sessionId = this.extractSessionId(this.currentPath);
          if (sessionId) {
            await invoke("create_encrypted_directory", {
              sessionId: sessionId,
              dirPath: newPath,
            });
          } else {
            throw new Error("Session chiffrée non trouvée");
          }
        } else {
          // Pour les dossiers normaux
          await invoke("create_directory", { dirPath: newPath });
        }
        await this.refresh();
      } catch (error) {
        console.error("Erreur lors de la création du dossier:", error);
        alert(`Erreur lors de la création du dossier: ${error}`);
      }
    },

    async uploadFile() {
      try {
        // Créer un input file caché
        const input = document.createElement("input");
        input.type = "file";
        input.multiple = false;

        input.onchange = async (event) => {
          const file = event.target.files[0];
          if (!file) return;

          const filePath = this.currentPath.endsWith("/")
            ? `${this.currentPath}${file.name}`
            : `${this.currentPath}/${file.name}`;

          try {
            if (this.partitionType === "encrypted") {
              // Pour les fichiers chiffrés
              const sessionId = this.extractSessionId(this.currentPath);
              if (sessionId) {
                const arrayBuffer = await file.arrayBuffer();
                const content = new Uint8Array(arrayBuffer);
                await invoke("upload_encrypted_file", {
                  sessionId: sessionId,
                  filePath: filePath,
                  content: Array.from(content),
                });
              } else {
                throw new Error("Session chiffrée non trouvée");
              }
            } else {
              // Pour les fichiers normaux
              const arrayBuffer = await file.arrayBuffer();
              const content = new Uint8Array(arrayBuffer);
              await invoke("write_file", {
                filePath: filePath,
                content: Array.from(content),
              });
            }
            await this.refresh();
            alert(`Fichier "${file.name}" uploadé avec succès !`);
          } catch (error) {
            console.error("Erreur lors de l'upload:", error);
            alert(`Erreur lors de l'upload: ${error}`);
          }
        };

        input.click();
      } catch (error) {
        console.error("Erreur lors de l'upload:", error);
        alert(`Erreur lors de l'upload: ${error}`);
      }
    },

    async downloadFile() {
      if (!this.editingFile) return;

      try {
        const content = await invoke("read_file", {
          filePath: this.editingFile.path,
        });
        const blob = new Blob([new Uint8Array(content)]);
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = this.editingFile.name;
        a.click();
        URL.revokeObjectURL(url);
      } catch (error) {
        console.error("Erreur lors du téléchargement:", error);
      }
    },

    async goUp() {
      if (!this.canGoUp) return;

      const parentPath =
        this.currentPath.split("/").slice(0, -1).join("/") || "/";
      this.currentPath = parentPath;

      if (this.partitionType === "public") {
        await this.loadFiles(this.currentPath);
      } else {
        const sessionId = this.extractSessionId(this.currentPath);
        if (sessionId) {
          await this.loadEncryptedFiles(sessionId, this.currentPath);
        }
      }
    },

    closeEditor() {
      this.showEditor = false;
      this.editingFile = null;
      this.fileContent = "";
    },

    isTextFile(filename) {
      const textExtensions = [
        ".txt",
        ".md",
        ".json",
        ".xml",
        ".html",
        ".css",
        ".js",
        ".py",
        ".rs",
        ".toml",
      ];
      return textExtensions.some((ext) => filename.toLowerCase().endsWith(ext));
    },

    getFileIcon(filename) {
      const ext = filename.split(".").pop()?.toLowerCase();
      const iconMap = {
        txt: "📄",
        md: "📝",
        json: "📋",
        xml: "📄",
        html: "🌐",
        css: "🎨",
        js: "📜",
        py: "🐍",
        rs: "🦀",
        jpg: "🖼️",
        jpeg: "🖼️",
        png: "🖼️",
        gif: "🖼️",
        pdf: "📕",
        zip: "📦",
        rar: "📦",
        exe: "⚙️",
        dll: "⚙️",
      };
      return iconMap[ext] || "📄";
    },

    formatFileSize(bytes) {
      if (bytes === 0) return "0 B";
      const k = 1024;
      const sizes = ["B", "KB", "MB", "GB", "TB"];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    },

    formatDate(timestamp) {
      return new Date(timestamp * 1000).toLocaleString();
    },

    async closeExplorer() {
      // Si c'est une partition chiffrée avec session, la fermer
      if (
        this.partitionType === "encrypted" &&
        this.partitionInfo?.partition?.sessionId
      ) {
        try {
          await invoke("close_encrypted_session", {
            sessionId: this.partitionInfo.partition.sessionId,
          });
          console.log("Session chiffrée fermée");
        } catch (error) {
          console.warn("Erreur lors de la fermeture de la session:", error);
        }
      }
      this.$emit("close-explorer");
    },
  },
};
</script>

<style scoped>
.file-explorer {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #1a1a1a;
  color: #ffffff;
}

.header {
  padding: 1rem;
  border-bottom: 1px solid #333;
  background: #2a2a2a;
}

.header-top {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.header h2 {
  margin: 0;
  color: #00ff88;
}

.close-explorer-btn {
  background: #666;
  color: #fff;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
}

.close-explorer-btn:hover {
  background: #777;
}

.partition-info {
  background: #333;
  padding: 0.75rem;
  border-radius: 4px;
  margin-bottom: 1rem;
}

.partition-info p {
  margin: 0.25rem 0;
  color: #ccc;
  font-size: 0.9rem;
}

.path-bar {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.current-path {
  font-family: monospace;
  background: #333;
  padding: 0.5rem;
  border-radius: 4px;
  flex: 1;
}

.up-button {
  background: #00ff88;
  color: #000;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
}

.up-button:disabled {
  background: #666;
  cursor: not-allowed;
}

.password-section {
  display: flex;
  justify-content: center;
  align-items: center;
  flex: 1;
}

.password-form {
  text-align: center;
  background: #2a2a2a;
  padding: 2rem;
  border-radius: 8px;
  border: 1px solid #333;
}

.password-form h3 {
  margin: 0 0 1rem 0;
  color: #00ff88;
}

.password-input {
  width: 300px;
  padding: 0.75rem;
  margin: 0.5rem;
  border: 1px solid #555;
  border-radius: 4px;
  background: #1a1a1a;
  color: #fff;
  font-size: 1rem;
}

.auth-button {
  background: #00ff88;
  color: #000;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
  margin: 0.5rem;
}

.auth-button:disabled {
  background: #666;
  cursor: not-allowed;
}

.error {
  color: #ff4444;
  margin-top: 1rem;
}

.file-content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.toolbar {
  padding: 1rem;
  border-bottom: 1px solid #333;
  display: flex;
  gap: 1rem;
}

.toolbar button {
  background: #333;
  color: #fff;
  border: 1px solid #555;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
}

.toolbar button:hover {
  background: #444;
}

.file-list {
  flex: 1;
  overflow-y: auto;
}

.file-item {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #333;
  cursor: pointer;
  align-items: center;
}

.file-item:hover {
  background: #2a2a2a;
}

.file-item.header {
  background: #333;
  font-weight: bold;
  cursor: default;
}

.file-item.header:hover {
  background: #333;
}

.file-item.directory {
  font-weight: bold;
}

.file-name {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.file-icon {
  font-size: 1.2rem;
}

.file-size,
.file-modified {
  font-family: monospace;
  font-size: 0.9rem;
  color: #aaa;
}

.file-actions {
  display: flex;
  gap: 0.5rem;
}

.action-button {
  background: #444;
  color: #fff;
  border: 1px solid #555;
  padding: 0.25rem 0.5rem;
  border-radius: 3px;
  cursor: pointer;
  font-size: 0.8rem;
}

.action-button:hover {
  background: #555;
}

.action-button.delete {
  background: #ff4444;
  border-color: #ff6666;
}

.action-button.delete:hover {
  background: #ff6666;
}

.empty-state {
  text-align: center;
  padding: 2rem;
  color: #666;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background: #2a2a2a;
  border: 1px solid #333;
  border-radius: 8px;
  width: 80%;
  max-width: 800px;
  max-height: 80%;
  display: flex;
  flex-direction: column;
}

.modal-header {
  padding: 1rem;
  border-bottom: 1px solid #333;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h3 {
  margin: 0;
  color: #00ff88;
}

.close-button {
  background: none;
  border: none;
  color: #fff;
  font-size: 1.5rem;
  cursor: pointer;
}

.modal-body {
  flex: 1;
  padding: 1rem;
  overflow: auto;
}

.file-editor {
  width: 100%;
  height: 400px;
  background: #1a1a1a;
  color: #fff;
  border: 1px solid #555;
  border-radius: 4px;
  padding: 1rem;
  font-family: monospace;
  font-size: 14px;
  resize: vertical;
}

.binary-file {
  text-align: center;
  padding: 2rem;
}

.download-button {
  background: #00ff88;
  color: #000;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
  margin-top: 1rem;
}

.modal-footer {
  padding: 1rem;
  border-top: 1px solid #333;
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
}

.save-button {
  background: #00ff88;
  color: #000;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
}

.cancel-button {
  background: #666;
  color: #fff;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 4px;
  cursor: pointer;
}
</style>
