<template>
  <div class="config-type">
    <div class="screen-header">
      <h2>Configurer la clé sélectionnée</h2>
    </div>

    <div class="config-options">
      <div
        class="option-card"
        :class="{ selected: selectedType === 'container' }"
        @click="selectType('container')"
      >
        <div class="option-header">
          <input
            type="radio"
            :value="'container'"
            v-model="selectedType"
            class="radio-input"
          />
          <h3>Conteneur chiffré sur partition publique</h3>
        </div>
        <p class="option-description">Recommandé, portable</p>
        <ul class="option-features">
          <li>✓ Compatible avec tous les systèmes</li>
          <li>✓ Facile à transporter</li>
          <li>✓ Formatage normal n'affecte pas les données</li>
        </ul>
      </div>

      <div
        class="option-card"
        :class="{ selected: selectedType === 'partition' }"
        @click="selectType('partition')"
      >
        <div class="option-header">
          <input
            type="radio"
            :value="'partition'"
            v-model="selectedType"
            class="radio-input"
          />
          <h3>Partition chiffrée séparée</h3>
        </div>
        <p class="option-description">
          Plus discret, perte si repartitionnement
        </p>
        <ul class="option-features">
          <li>✓ Plus discret</li>
          <li>✓ Performance optimale</li>
          <li>⚠ Risque de perte lors du repartitionnement</li>
        </ul>
      </div>

      <div
        class="option-card"
        :class="{ selected: selectedType === 'hidden' }"
        @click="selectType('hidden')"
      >
        <div class="option-header">
          <input
            type="radio"
            :value="'hidden'"
            v-model="selectedType"
            class="radio-input"
          />
          <h3>Volume caché (VeraCrypt)</h3>
        </div>
        <p class="option-description">Plausible deniability</p>
        <ul class="option-features">
          <li>✓ Déni plausible</li>
          <li>✓ Double protection</li>
          <li>⚠ Complexité accrue</li>
        </ul>
      </div>
    </div>

    <div class="info-box">
      <div class="warning-icon">⚠</div>
      <div class="warning-content">
        <p><strong>Droits administrateur requis</strong></p>
        <p>
          Cette opération nécessite des privilèges élevés pour manipuler les
          partitions du périphérique.
        </p>
      </div>
    </div>

    <div class="actions">
      <button @click="goBack" class="cancel-btn">Annuler</button>
      <button
        @click="proceedToSecurity"
        class="next-btn"
        :disabled="!selectedType"
      >
        Suivant
      </button>
    </div>
  </div>
</template>

<script>
import { ref } from "vue";
import { useRouter } from "vue-router";

export default {
  name: "ConfigType",
  setup() {
    const router = useRouter();
    const selectedType = ref("");

    const selectType = (type) => {
      selectedType.value = type;
    };

    const goBack = () => {
      router.push("/");
    };

    const proceedToSecurity = () => {
      if (selectedType.value) {
        router.push("/security");
      }
    };

    return {
      selectedType,
      selectType,
      goBack,
      proceedToSecurity,
    };
  },
};
</script>

<style scoped>
.config-type {
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

.config-options {
  display: grid;
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.option-card {
  background: #161b22;
  border: 2px solid #30363d;
  border-radius: 8px;
  padding: 1.5rem;
  cursor: pointer;
  transition: all 0.2s ease;
}

.option-card:hover {
  border-color: #58a6ff;
  background: #21262d;
}

.option-card.selected {
  border-color: #238636;
  background: #0d1117;
  box-shadow: 0 0 0 1px #238636;
}

.option-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.5rem;
}

.radio-input {
  width: 18px;
  height: 18px;
  accent-color: #238636;
}

.option-header h3 {
  color: #f0f6fc;
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
}

.option-description {
  color: #7d8590;
  margin: 0 0 1rem 0;
  font-style: italic;
}

.option-features {
  list-style: none;
  padding: 0;
  margin: 0;
}

.option-features li {
  color: #c9d1d9;
  margin: 0.5rem 0;
  font-size: 0.9rem;
}

.info-box {
  background: #1a1a1a;
  border: 1px solid #f85149;
  border-radius: 6px;
  padding: 1rem;
  display: flex;
  gap: 1rem;
  margin-bottom: 2rem;
}

.warning-icon {
  color: #f85149;
  font-size: 1.5rem;
  font-weight: bold;
}

.warning-content p {
  margin: 0;
  color: #c9d1d9;
}

.warning-content p:first-child {
  font-weight: 600;
  color: #f85149;
}

.actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
}

.cancel-btn,
.next-btn {
  padding: 0.75rem 2rem;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  font-size: 1rem;
  transition: all 0.2s ease;
}

.cancel-btn {
  background: transparent;
  color: #c9d1d9;
  border: 1px solid #30363d;
}

.cancel-btn:hover {
  background: #21262d;
  border-color: #58a6ff;
}

.next-btn {
  background: #238636;
  color: white;
  border: none;
}

.next-btn:hover:not(:disabled) {
  background: #2ea043;
}

.next-btn:disabled {
  background: #484f58;
  cursor: not-allowed;
  opacity: 0.5;
}
</style>
