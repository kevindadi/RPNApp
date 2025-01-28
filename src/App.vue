<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { Splitpanes, Pane } from 'splitpanes'
import 'splitpanes/dist/splitpanes.css'
import { invoke } from '@tauri-apps/api/tauri'

const sourceCode = ref('')
const irCode = ref('')
const graphContent = ref('')
const info = ref('')

// 添加按钮状态控制
const isProcessing = reactive({
  mir: false,
  petri: false
})

// 添加标题和介绍
const pageInfo = {
  title: "Rust并发程序分析工具",
  description: "这是一个基于 Petri 网的 Rust 并发程序分析工具，可以检测死锁、原子性违反和数据竞争等并发问题。"
}

// 添加检测模式配置
const detectionModes = [
  { id: 'deadlock', name: '死锁检测', flag: '-m deadlock' },
  { id: 'atomicity', name: '原子性违反检测', flag: '-m atomic' },
  { id: 'race', name: '数据竞争检测', flag: '-m datarace' }
]
const selectedMode = ref('deadlock')


// 面板状态管理
const panels = reactive({
  sourceCode: { visible: true, size: 25 },
  irCode: { visible: true, size: 25 },
  graph: { visible: true, size: 25 },
  info: { visible: true, size: 25 }
})

// 图片缩放和拖动相关状态
const imageStyle = reactive({
  transform: 'scale(1) translate(0px, 0px)',
  cursor: 'grab'
})
const isDragging = ref(false)
const dragStart = reactive({ x: 0, y: 0 })
const currentTranslate = reactive({ x: 0, y: 0 })
const scale = ref(1)

// 添加示例配置
const examples = [
  { name: 'Example 1', file: 'example1.rs' },
  { name: 'Example 2', file: 'example2.rs' },
  { name: 'Example 3', file: 'example3.rs' }
]

const selectedExample = ref('')

// 面板控制函数
function togglePanel(panelKey: string) {
  const panel = panels[panelKey as keyof typeof panels]
  panel.visible = !panel.visible
  redistributeSizes()
}

function redistributeSizes() {
  const visiblePanels = Object.values(panels).filter(p => p.visible).length
  if (visiblePanels > 0) {
    const newSize = 100 / visiblePanels
    Object.values(panels).forEach(panel => {
      if (panel.visible) {
        panel.size = newSize
      }
    })
  }
}

function getPanelName(key: string): string {
  const names = {
    sourceCode: '源代码',
    irCode: '中间代码',
    graph: '图形',
    info: '信息'
  }
  return names[key as keyof typeof names]
}

// 图片缩放和拖动处理函数
function handleZoom(e: WheelEvent) {
  e.preventDefault()
  const delta = e.deltaY > 0 ? 0.9 : 1.1
  scale.value = Math.min(Math.max(0.1, scale.value * delta), 5)
  updateImageTransform()
}

function startDrag(e: MouseEvent) {
  isDragging.value = true
  dragStart.x = e.clientX - currentTranslate.x
  dragStart.y = e.clientY - currentTranslate.y
  imageStyle.cursor = 'grabbing'
}

function onDrag(e: MouseEvent) {
  if (!isDragging.value) return
  currentTranslate.x = e.clientX - dragStart.x
  currentTranslate.y = e.clientY - dragStart.y
  updateImageTransform()
}

function stopDrag() {
  isDragging.value = false
  imageStyle.cursor = 'grab'
}

function updateImageTransform() {
  imageStyle.transform = `scale(${scale.value}) translate(${currentTranslate.x}px, ${currentTranslate.y}px)`
}

function onResize() {
  // 处理面板大小调整后的逻辑
}

// 分离 MIR 生成函数
async function handleGenerateMir() {
  if (!sourceCode.value.trim()) {
    info.value = "请先输入源代码或选择示例"
    return
  }
  
  try {
    isProcessing.mir = true
    irCode.value = await invoke('generate_mir', { sourceCode: sourceCode.value })
  } catch (error) {
    console.error('MIR 生成失败:', error)
    irCode.value = `Error: ${error}`
  } finally {
    isProcessing.mir = false
  }
}

// 分离 Petri 网分析函数
async function handlePetriAnalysis() {
  if (!sourceCode.value.trim()) {
    info.value = "请先输入源代码或选择示例"
    return
  }
  
  try {
    isProcessing.petri = true
    
    const mode = detectionModes.find(m => m.id === selectedMode.value)
    const result = await invoke('run_pn_analysis', { sourceCode: sourceCode.value, mode: mode?.flag || '-m deadlock' })
    
    if (typeof result === 'object' && result) {
      const { graphContent: svg, output, error } = result as { 
        graphContent: string, 
        output: string, 
        error: string 
      }
      graphContent.value = svg
      
      // 格式化输出内容
      const analysisMode = mode?.name || '死锁检测'
      const outputLines = output.split('\n').filter(line => line.trim())
      
      info.value = `分析模式: ${analysisMode}\n` +
        '----------------------------------------\n' +
        '分析结果:\n' +
        outputLines
          .map(line => line.trim())
          .filter(line => !line.startsWith('==='))  // 过滤掉分隔符行
          .join('\n') +
        '\n----------------------------------------\n'
    }
  } catch (error) {
    console.error('Petri 网分析失败:', error)
    info.value = `错误: ${error}`
  } finally {
    isProcessing.petri = false
  }
}

// 修改 loadExample 函数，只加载源代码
async function loadExample(filename: string) {
  try {
    console.log('Loading example file:', filename)
    const code = await invoke('read_example', { filename })
    if (typeof code === 'string') {
      sourceCode.value = code
    } else {
      console.error('Unexpected response type:', typeof code)
    }
  } catch (error) {
    console.error('加载示例失败:', error)
    info.value = `Error: ${error}`
  }
}

// 修改 handleExampleChange 函数
async function handleExampleChange(event: Event) {
  const target = event.target as HTMLSelectElement
  if (target.value) {
    console.log('Selected example:', target.value)
    // 直接加载示例，不要清空 sourceCode
    await loadExample(target.value)
  }
}
</script>

<template>
  <div class="container">
    <div class="header">
      <h1>{{ pageInfo.title }}</h1>
      <p class="description">{{ pageInfo.description }}</p>
      <div class="controls">
        <!-- 示例选择 -->
        <select 
          v-model="selectedExample"
          @change="handleExampleChange"
          class="example-select"
        >
          <option value="">选择示例...</option>
          <option 
            v-for="example in examples" 
            :key="example.file" 
            :value="example.file"
          >
            {{ example.name }}
          </option>
        </select>
        
        <!-- 分析模式选择 -->
        <div class="mode-selector">
          <label>检测模式：</label>
          <select v-model="selectedMode">
            <option 
              v-for="mode in detectionModes" 
              :key="mode.id" 
              :value="mode.id"
            >
              {{ mode.name }}
            </option>
          </select>
        </div>
        
        <!-- 添加操作按钮 -->
        <div class="action-buttons">
          <button 
            @click="handleGenerateMir" 
            :disabled="isProcessing.mir"
            class="action-btn"
          >
            {{ isProcessing.mir ? '生成中...' : '生成 MIR' }}
          </button>
          <button 
            @click="handlePetriAnalysis" 
            :disabled="isProcessing.petri"
            class="action-btn"
          >
            {{ isProcessing.petri ? '分析中...' : 'Petri 网分析' }}
          </button>
        </div>
      </div>
    </div>

    <Splitpanes class="default-theme" @resize="onResize">
      <Pane v-if="panels.sourceCode.visible" :size="panels.sourceCode.size">
        <div class="panel source-code">
          <div class="panel-header">
            <h3>RustSourceCode</h3>
          </div>
          <textarea
            v-model="sourceCode"
            placeholder="在此粘贴源代码或选择示例..."
            class="code-input"
          ></textarea>
        </div>
      </Pane>

      <Pane v-if="panels.irCode.visible" :size="panels.irCode.size">
        <div class="panel ir-code">
          <div class="panel-header">
            <h3>MIR</h3>
            <button @click="togglePanel('irCode')" class="close-btn">×</button>
          </div>
          <pre class="ir-display">{{ irCode }}</pre>
        </div>
      </Pane>

      <Pane v-if="panels.graph.visible" :size="panels.graph.size">
        <div class="panel graph">
          <div class="panel-header">
            <h3>PetriNet</h3>
            <button @click="togglePanel('graph')" class="close-btn">×</button>
          </div>
          <div class="graph-container">
            <!-- 使用 v-html 直接插入 SVG 内容 -->
            <div v-html="graphContent" class="svg-wrapper"></div>
          </div>
        </div>
      </Pane>

      <Pane v-if="panels.info.visible" :size="panels.info.size">
        <div class="panel info">
          <div class="panel-header">
            <h3>Detecion</h3>
            <button @click="togglePanel('info')" class="close-btn">×</button>
          </div>
          <div class="info-content">{{ info }}</div>
        </div>
      </Pane>
    </Splitpanes>

    <div class="panel-controls">
      <button
        v-for="(panel, key) in panels"
        :key="key"
        @click="togglePanel(key)"
        :class="{ active: panel.visible }"
      >
        {{ getPanelName(key) }}
      </button>
    </div>
  </div>
</template>

<style scoped>

/* 添加新的样式 */
.header {
  padding: 16px;
  background: #f8f9fa;
  border-bottom: 1px solid #ddd;
}

.header h1 {
  margin: 0;
  font-size: 24px;
  color: #333;
}

.description {
  margin: 8px 0;
  color: #666;
  font-size: 14px;
}

.controls {
  display: flex;
  gap: 16px;
  align-items: center;
  margin-top: 12px;
  flex-wrap: wrap;
}
.mode-selector {
  display: flex;
  align-items: center;
  gap: 8px;
}

.mode-selector label {
  font-size: 14px;
  color: #555;
}

.mode-selector select,
.example-select {
  padding: 4px 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  font-size: 14px;
  min-width: 120px;
}

.container {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.splitpanes {
  height: calc(100% - 40px - 120px);
}

.panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #f5f5f5;
  border: 1px solid #ddd;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  background: #e0e0e0;
}

.close-btn {
  border: none;
  background: none;
  font-size: 20px;
  cursor: pointer;
  padding: 0 8px;
}

.code-input {
  flex: 1;
  width: 100%;
  padding: 8px;
  font-family: monospace;
  resize: none;
  border: none;
  background: #fff;
}

.ir-display {
  flex: 1;
  padding: 8px;
  margin: 0;
  overflow: auto;
  background: #fff;
  white-space: pre-wrap;
}

.info-content {
  flex: 1;
  padding: 8px;
  overflow: auto;
  background: #fff;
}

.panel-controls {
  height: 40px;
  display: flex;
  gap: 8px;
  padding: 8px;
  background: #f0f0f0;
  border-top: 1px solid #ddd;
}

.panel-controls button {
  padding: 4px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: #fff;
  cursor: pointer;
}

.panel-controls button.active {
  background: #e0e0e0;
}

.example-select {
  padding: 4px 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  font-size: 14px;
}

.graph-container {
  flex: 1;
  overflow: auto;
  background: white;
  padding: 16px;
  display: flex;
  justify-content: center;
  align-items: center;
}

.svg-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
}

.svg-wrapper :deep(svg) {
  max-width: 100%;
  max-height: 100%;
}

/* 添加新的按钮样式 */
.action-buttons {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: #fff;
  cursor: pointer;
  transition: all 0.3s;
  min-width: 100px;
}

.action-btn:hover:not(:disabled) {
  background: #f0f0f0;
}

.action-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
