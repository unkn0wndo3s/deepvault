<template>
  <div class="security-config">
    <div class="screen-header">
      <h2>Paramètres de sécurité</h2>
    </div>

    <div class="config-form">
      <div class="form-group">
        <label for="password">Mot de passe</label>
        <input
          type="password"
          id="password"
          v-model="password"
          placeholder="Entrez votre mot de passe"
          class="form-input"
          :class="{ error: passwordError }"
        />
        <div v-if="passwordError" class="error-message">
          {{ passwordError }}
        </div>
      </div>

      <div class="form-group">
        <label for="confirmPassword">Confirmer mot de passe</label>
        <input
          type="password"
          id="confirmPassword"
          v-model="confirmPassword"
          placeholder="Confirmez votre mot de passe"
          class="form-input"
          :class="{ error: confirmPasswordError }"
        />
        <div v-if="confirmPasswordError" class="error-message">
          {{ confirmPasswordError }}
        </div>
      </div>

      <div class="form-group">
        <label>Algorithme KDF</label>
        <div class="algorithm-info">
          <span class="algorithm-name">Argon2id (recommandé)</span>
          <span class="algorithm-desc"
            >Résistant aux attaques par canal auxiliaire</span
          >
        </div>
      </div>

      <div class="advanced-section">
        <h3>Paramètres avancés</h3>
        <div class="advanced-grid">
          <div class="form-group">
            <label for="memory">Mémoire (MB)</label>
            <input
              type="number"
              id="memory"
              v-model.number="kdfParams.memory"
              min="1"
              max="1024"
              class="form-input"
            />
          </div>

          <div class="form-group">
            <label for="iterations">Itérations</label>
            <input
              type="number"
              id="iterations"
              v-model.number="kdfParams.iterations"
              min="1"
              max="100"
              class="form-input"
            />
          </div>

          <div class="form-group">
            <label for="parallelism">Parallélisme</label>
            <input
              type="number"
              id="parallelism"
              v-model.number="kdfParams.parallelism"
              min="1"
              max="8"
              class="form-input"
            />
          </div>
        </div>
      </div>

      <div class="form-group">
        <label for="headerFile">Nom du fichier header metadata</label>
        <input
          type="text"
          id="headerFile"
          v-model="headerFile"
          placeholder=".dv_meta"
          class="form-input"
        />
      </div>

      <div class="checkbox-group">
        <label class="checkbox-label">
          <input
            type="checkbox"
            v-model="enableHiddenVolume"
            class="checkbox-input"
          />
          <span class="checkbox-text"
            >Activer volume caché (si prise en charge)</span
          >
        </label>
      </div>
    </div>

    <div class="info-box">
      <div class="info-icon">ℹ</div>
      <div class="info-content">
        <p><strong>Dérivation de clé PKDF</strong></p>
        <p>
          Le sel sera généré aléatoirement et stocké dans la partition publique
          sous un nom non-suspect.
        </p>
        <p>
          Mémoire: {{ kdfParams.memory }}MB | Itérations:
          {{ kdfParams.iterations }} | Parallélisme: {{ kdfParams.parallelism }}
        </p>
      </div>
    </div>

    <div class="actions">
      <button @click="goBack" class="back-btn">Retour</button>
      <button
        @click="createVolume"
        class="create-btn"
        :disabled="!isFormValid || creating"
      >
        {{ creating ? "Création..." : "Créer" }}
      </button>
    </div>
  </div>
</template>

<script>
import { ref, computed, watch } from "vue";
import { useRouter } from "vue-router";

export default {
  name: "SecurityConfig",
  setup() {
    const router = useRouter();
    const password = ref("");
    const confirmPassword = ref("");
    const passwordError = ref("");
    const confirmPasswordError = ref("");
    const headerFile = ref(".dv_meta");
    const enableHiddenVolume = ref(false);
    const creating = ref(false);

    const kdfParams = ref({
      memory: 64,
      iterations: 3,
      parallelism: 1,
    });

    const validatePassword = (pwd) => {
      if (pwd.length < 8) {
        return "Le mot de passe doit contenir au moins 8 caractères";
      }
      if (pwd.length > 128) {
        return "Le mot de passe ne peut pas dépasser 128 caractères";
      }
      if (!/[A-Z]/.test(pwd)) {
        return "Le mot de passe doit contenir au moins une majuscule";
      }
      if (!/[a-z]/.test(pwd)) {
        return "Le mot de passe doit contenir au moins une minuscule";
      }
      if (!/[0-9]/.test(pwd)) {
        return "Le mot de passe doit contenir au moins un chiffre";
      }
      if (!/[!@#$%^&*()_+\-=\[\]{}|;:,.<>?]/.test(pwd)) {
        return "Le mot de passe doit contenir au moins un caractère spécial";
      }
      return "";
    };

    const validateConfirmPassword = () => {
      if (confirmPassword.value && password.value !== confirmPassword.value) {
        return "Les mots de passe ne correspondent pas";
      }
      return "";
    };

    watch(password, (newPassword) => {
      passwordError.value = validatePassword(newPassword);
    });

    watch(confirmPassword, () => {
      confirmPasswordError.value = validateConfirmPassword();
    });

    const isFormValid = computed(() => {
      return (
        password.value &&
        confirmPassword.value &&
        !passwordError.value &&
        !confirmPasswordError.value &&
        password.value === confirmPassword.value
      );
    });

    const goBack = () => {
      router.push("/config-type");
    };

    const createVolume = async () => {
      if (!isFormValid.value) return;

      creating.value = true;

      try {
        // TODO: Implement volume creation
        console.log("Creating volume with config:", {
          password: password.value,
          kdfParams: kdfParams.value,
          headerFile: headerFile.value,
          enableHiddenVolume: enableHiddenVolume.value,
        });

        // Simulate creation process
        await new Promise((resolve) => setTimeout(resolve, 2000));

        router.push("/progress");
      } catch (error) {
        console.error("Error creating volume:", error);
      } finally {
        creating.value = false;
      }
    };

    return {
      password,
      confirmPassword,
      passwordError,
      confirmPasswordError,
      kdfParams,
      headerFile,
      enableHiddenVolume,
      creating,
      isFormValid,
      goBack,
      createVolume,
    };
  },
};
</script>

<style scoped>
.security-config {
  max-width: 600px;
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

.config-form {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 2rem;
  margin-bottom: 2rem;
}

.form-group {
  margin-bottom: 1.5rem;
}

.form-group label {
  display: block;
  color: #f0f6fc;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.form-input {
  width: 100%;
  padding: 0.75rem;
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  color: #c9d1d9;
  font-size: 1rem;
}

.form-input:focus {
  outline: none;
  border-color: #58a6ff;
  box-shadow: 0 0 0 2px rgba(88, 166, 255, 0.3);
}

.form-input.error {
  border-color: #f85149;
}

.error-message {
  color: #f85149;
  font-size: 0.875rem;
  margin-top: 0.25rem;
}

.algorithm-info {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  padding: 1rem;
}

.algorithm-name {
  display: block;
  color: #3fb950;
  font-weight: 600;
  margin-bottom: 0.25rem;
}

.algorithm-desc {
  color: #7d8590;
  font-size: 0.9rem;
}

.advanced-section {
  margin-top: 2rem;
  padding-top: 2rem;
  border-top: 1px solid #30363d;
}

.advanced-section h3 {
  color: #f0f6fc;
  margin: 0 0 1rem 0;
  font-size: 1.1rem;
}

.advanced-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 1rem;
}

.checkbox-group {
  margin-top: 1.5rem;
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

.info-box {
  background: #0d1117;
  border: 1px solid #58a6ff;
  border-radius: 6px;
  padding: 1rem;
  display: flex;
  gap: 1rem;
  margin-bottom: 2rem;
}

.info-icon {
  color: #58a6ff;
  font-size: 1.5rem;
  font-weight: bold;
}

.info-content p {
  margin: 0;
  color: #c9d1d9;
  font-size: 0.9rem;
}

.info-content p:first-child {
  font-weight: 600;
  color: #58a6ff;
}

.actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
}

.back-btn,
.create-btn {
  padding: 0.75rem 2rem;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  font-size: 1rem;
  transition: all 0.2s ease;
}

.back-btn {
  background: transparent;
  color: #c9d1d9;
  border: 1px solid #30363d;
}

.back-btn:hover {
  background: #21262d;
  border-color: #58a6ff;
}

.create-btn {
  background: #238636;
  color: white;
  border: none;
}

.create-btn:hover:not(:disabled) {
  background: #2ea043;
}

.create-btn:disabled {
  background: #484f58;
  cursor: not-allowed;
  opacity: 0.5;
}
</style>
