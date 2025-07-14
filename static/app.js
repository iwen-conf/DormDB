// DormDB 前端应用 - Vue 3 + Tailwind CSS + Font Awesome
// 现代化的响应式设计和用户体验

const { createApp, ref, reactive, computed, onMounted, nextTick, watch } = Vue;

// 配置 Axios
axios.defaults.withCredentials = true;
axios.defaults.timeout = 10000;

// 添加请求拦截器
axios.interceptors.request.use(
    config => {
        return config;
    },
    error => {
        return Promise.reject(error);
    }
);

// 添加响应拦截器
axios.interceptors.response.use(
    response => {
        return response;
    },
    error => {
        console.error('请求错误:', error);
        return Promise.reject(error);
    }
);

// 全局配置
const APP_CONFIG = {
    API_BASE_URL: '/api/v1',
    TOAST_DURATION: 3000,
    ANIMATION_DURATION: 300,
    DEBOUNCE_DELAY: 500
};

// 工具函数
const utils = {
    // 防抖函数
    debounce(func, wait) {
        let timeout;
        return function executedFunction(...args) {
            const later = () => {
                clearTimeout(timeout);
                func(...args);
            };
            clearTimeout(timeout);
            timeout = setTimeout(later, wait);
        };
    },
    
    // 格式化日期时间
    formatDateTime(dateString) {
        const date = new Date(dateString);
        return date.toLocaleString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit'
        });
    },
    
    // 验证学号格式
    validateStudentId(studentId) {
        return /^[0-9]{10}$/.test(studentId);
    },
    
    // 生成随机ID
    generateId() {
        return Date.now().toString(36) + Math.random().toString(36).substr(2);
    },
    
    // 复制到剪贴板
    async copyToClipboard(text) {
        try {
            await navigator.clipboard.writeText(text);
            return true;
        } catch (err) {
            // 降级方案
            const textArea = document.createElement('textarea');
            textArea.value = text;
            document.body.appendChild(textArea);
            textArea.select();
            document.execCommand('copy');
            document.body.removeChild(textArea);
            return true;
        }
    },
    
    // 下载文件
    downloadFile(content, filename, contentType = 'text/plain') {
        const blob = new Blob([content], { type: contentType });
        const url = window.URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        window.URL.revokeObjectURL(url);
    }
};

// API 服务
const apiService = {
    // 申请数据库
    async applyDatabase(studentId) {
        try {
            const response = await axios.post(`${APP_CONFIG.API_BASE_URL}/apply`, {
                student_id: studentId
            });
            return response.data;
        } catch (error) {
            console.error('申请数据库失败:', error);
            throw new Error('网络连接错误，请检查网络后重试');
        }
    },
    
    // 查询申请状态
    async queryStatus(studentId) {
        try {
            const response = await axios.get(`${APP_CONFIG.API_BASE_URL}/query/${studentId}`);
            return response.data;
        } catch (error) {
            console.error('查询状态失败:', error);
            throw new Error('网络连接错误，请检查网络后重试');
        }
    },
    
    // 获取系统统计
    async getStats() {
        try {
            const response = await axios.get(`${APP_CONFIG.API_BASE_URL}/stats`);
            return response.data;
        } catch (error) {
            console.error('获取统计失败:', error);
            throw new Error('网络连接错误，请检查网络后重试');
        }
    },
    
    // 获取最近申请记录
    async getRecentApplications(limit = 10) {
        try {
            const response = await axios.get(`${APP_CONFIG.API_BASE_URL}/recent?limit=${limit}`);
            return response.data;
        } catch (error) {
            console.error('获取最近申请记录失败:', error);
            throw new Error('网络连接错误，请检查网络后重试');
        }
    },
    
    // 检查系统状态
    async checkHealth() {
        try {
            const response = await axios.get(`${APP_CONFIG.API_BASE_URL}/health`);
            return response.data;
        } catch (error) {
            console.error('健康检查失败:', error);
            throw new Error('网络连接错误，请检查网络后重试');
        }
    }
};

// Toast 通知组件
const ToastManager = {
    toasts: ref([]),
    toastId: 0,
    
    show(type, title, message = '', duration = APP_CONFIG.TOAST_DURATION) {
        const toast = {
            id: ++this.toastId,
            type,
            title,
            message,
            timestamp: Date.now()
        };
        
        this.toasts.value.push(toast);
        
        setTimeout(() => {
            this.remove(toast.id);
        }, duration);
        
        return toast.id;
    },
    
    remove(id) {
        const index = this.toasts.value.findIndex(toast => toast.id === id);
        if (index > -1) {
            this.toasts.value.splice(index, 1);
        }
    },
    
    clear() {
        this.toasts.value = [];
    },
    
    success(title, message) {
        return this.show('success', title, message);
    },
    
    error(title, message) {
        return this.show('error', title, message);
    },
    
    warning(title, message) {
        return this.show('warning', title, message);
    },
    
    info(title, message) {
        return this.show('info', title, message);
    }
};

// 主应用组件
const DormDBApp = {
    setup() {
        // 响应式状态
        const state = reactive({
            // 应用状态
            isLoading: false,
            isInitialized: false,
            currentView: 'home', // home, apply, query, about
            
            // 申请表单
            applyForm: {
                studentId: '',
                isSubmitting: false,
                errors: {}
            },
            
            // 查询表单
            queryForm: {
                studentId: '',
                isQuerying: false,
                result: null,
                errors: {}
            },
            
            // 系统数据
            systemStats: {
                totalApplications: 0,
                todayApplications: 0,
                successRate: 0,
                systemStatus: 'unknown'
            },
            
            // 最近申请记录
            recentApplications: [],
            
            // UI状态
            showMobileMenu: false,
            showConnectionInfo: false,
            darkMode: false
        });
        
        // 计算属性
        const computedProps = {
            // 当前视图的标题
            currentViewTitle: computed(() => {
                const titles = {
                    home: '欢迎使用 DormDB',
                    apply: '申请数据库',
                    query: '查询申请状态',
                    about: '关于系统'
                };
                return titles[state.currentView] || '未知页面';
            }),
            
            // 申请表单是否有效
            isApplyFormValid: computed(() => {
                return utils.validateStudentId(state.applyForm.studentId) && 
                       !state.applyForm.isSubmitting;
            }),
            
            // 查询表单是否有效
            isQueryFormValid: computed(() => {
                return utils.validateStudentId(state.queryForm.studentId) && 
                       !state.queryForm.isQuerying;
            }),
            
            // 系统状态指示器
            systemStatusIndicator: computed(() => {
                const status = state.systemStats.systemStatus;
                const indicators = {
                    healthy: { color: 'bg-green-400', text: '系统正常', icon: 'fa-check-circle' },
                    degraded: { color: 'bg-yellow-400', text: '性能降级', icon: 'fa-exclamation-triangle' },
                    unhealthy: { color: 'bg-red-400', text: '系统异常', icon: 'fa-times-circle' },
                    unknown: { color: 'bg-gray-400', text: '状态未知', icon: 'fa-question-circle' }
                };
                return indicators[status] || indicators.unknown;
            })
        };
        
        // 方法
        const methods = {
            // 初始化应用
            async initializeApp() {
                state.isLoading = true;
                
                try {
                    // 并行加载初始数据
                    await Promise.all([
                        this.loadSystemStats(),
                        this.loadRecentApplications(),
                        this.checkSystemHealth()
                    ]);
                    
                    state.isInitialized = true;
                    ToastManager.success('系统就绪', 'DormDB 已成功加载');
                } catch (error) {
                    console.error('应用初始化失败:', error);
                    ToastManager.error('初始化失败', '系统加载时出现错误，部分功能可能不可用');
                } finally {
                    state.isLoading = false;
                }
            },
            
            // 加载系统统计
            async loadSystemStats() {
                try {
                    const response = await apiService.getStats();
                    if (response.code === 0) {
                        Object.assign(state.systemStats, response.data);
                    } else {
                        throw new Error(response.message);
                    }
                } catch (error) {
                    console.error('加载系统统计失败:', error);
                    throw error;
                }
            },
            
            // 加载最近申请记录
            async loadRecentApplications() {
                try {
                    const response = await apiService.getRecentApplications(5);
                    if (response.code === 0) {
                        state.recentApplications = response.data;
                    } else {
                        throw new Error(response.message);
                    }
                } catch (error) {
                    console.error('加载最近申请记录失败:', error);
                    // 这个不是关键功能，不抛出错误
                }
            },
            
            // 检查系统健康状态
            async checkSystemHealth() {
                try {
                    const response = await apiService.checkHealth();
                    if (response.code === 0) {
                        state.systemStats.systemStatus = response.data.status;
                    } else {
                        state.systemStats.systemStatus = 'unhealthy';
                    }
                } catch (error) {
                    console.error('检查系统健康状态失败:', error);
                    state.systemStats.systemStatus = 'unknown';
                }
            },
            
            // 切换视图
            switchView(view) {
                if (state.currentView === view) return;
                
                state.currentView = view;
                state.showMobileMenu = false;
                
                // 重置表单状态
                this.resetForms();
                
                // 滚动到顶部
                window.scrollTo({ top: 0, behavior: 'smooth' });
            },
            
            // 重置表单
            resetForms() {
                state.applyForm.studentId = '';
                state.applyForm.errors = {};
                state.queryForm.studentId = '';
                state.queryForm.result = null;
                state.queryForm.errors = {};
            },
            
            // 验证学号输入
            validateStudentId(studentId, formType) {
                const errors = {};
                
                if (!studentId) {
                    errors.studentId = '请输入学号';
                } else if (!utils.validateStudentId(studentId)) {
                    errors.studentId = '请输入有效的10位学号';
                }
                
                if (formType === 'apply') {
                    state.applyForm.errors = errors;
                } else if (formType === 'query') {
                    state.queryForm.errors = errors;
                }
                
                return Object.keys(errors).length === 0;
            },
            
            // 申请数据库
            async applyDatabase() {
                const studentId = state.applyForm.studentId.trim();
                
                if (!this.validateStudentId(studentId, 'apply')) {
                    ToastManager.error('表单错误', '请检查输入的学号格式');
                    return;
                }
                
                state.applyForm.isSubmitting = true;
                
                try {
                    const response = await apiService.applyDatabase(studentId);
                    
                    if (response.code === 0) {
                        ToastManager.success('申请成功', '数据库申请已提交，请记录您的连接信息');
                        
                        // 显示连接信息
                        state.showConnectionInfo = true;
                        
                        // 更新系统统计
                        await this.loadSystemStats();
                        await this.loadRecentApplications();
                        
                        // 重置表单
                        state.applyForm.studentId = '';
                        
                        // 自动切换到查询页面
                        setTimeout(() => {
                            this.switchView('query');
                            state.queryForm.studentId = studentId;
                        }, 3000);
                    } else {
                        ToastManager.error('申请失败', response.message || '申请过程中出现错误');
                    }
                } catch (error) {
                    console.error('申请数据库失败:', error);
                    ToastManager.error('网络错误', '无法连接到服务器，请检查网络连接');
                } finally {
                    state.applyForm.isSubmitting = false;
                }
            },
            
            // 查询申请状态
            async queryStatus() {
                const studentId = state.queryForm.studentId.trim();
                
                if (!this.validateStudentId(studentId, 'query')) {
                    ToastManager.error('表单错误', '请检查输入的学号格式');
                    return;
                }
                
                state.queryForm.isQuerying = true;
                state.queryForm.result = null;
                
                try {
                    const response = await apiService.queryStatus(studentId);
                    
                    if (response.code === 0) {
                        state.queryForm.result = response.data;
                        ToastManager.success('查询成功', '已获取申请状态信息');
                    } else {
                        ToastManager.error('查询失败', response.message || '查询过程中出现错误');
                    }
                } catch (error) {
                    console.error('查询申请状态失败:', error);
                    ToastManager.error('网络错误', '无法连接到服务器，请检查网络连接');
                } finally {
                    state.queryForm.isQuerying = false;
                }
            },
            
            // 复制连接信息
            async copyConnectionInfo() {
                const result = state.queryForm.result;
                if (!result) return;
                
                const connectionInfo = `数据库连接信息：
主机: ${result.host}
端口: ${result.port}
数据库: ${result.database}
用户名: ${result.username}
密码: ${result.password}`;
                
                try {
                    await utils.copyToClipboard(connectionInfo);
                    ToastManager.success('复制成功', '连接信息已复制到剪贴板');
                } catch (error) {
                    ToastManager.error('复制失败', '无法复制到剪贴板');
                }
            },
            
            // 下载连接信息
            downloadConnectionInfo() {
                const result = state.queryForm.result;
                if (!result) return;
                
                const connectionInfo = `# DormDB 数据库连接信息
# 学号: ${state.queryForm.studentId}
# 申请时间: ${utils.formatDateTime(result.created_at)}

主机: ${result.host}
端口: ${result.port}
数据库: ${result.database}
用户名: ${result.username}
密码: ${result.password}

# 注意：请妥善保管您的数据库连接信息
# 如有问题，请联系系统管理员`;
                
                const filename = `dormdb_${state.queryForm.studentId}_${new Date().toISOString().split('T')[0]}.txt`;
                utils.downloadFile(connectionInfo, filename);
                
                ToastManager.success('下载成功', '连接信息文件已下载');
            },
            
            // 刷新数据
            async refreshData() {
                try {
                    await Promise.all([
                        this.loadSystemStats(),
                        this.loadRecentApplications(),
                        this.checkSystemHealth()
                    ]);
                    ToastManager.success('刷新成功', '数据已更新');
                } catch (error) {
                    ToastManager.error('刷新失败', '无法获取最新数据');
                }
            },
            
            // 切换移动端菜单
            toggleMobileMenu() {
                state.showMobileMenu = !state.showMobileMenu;
            },
            
            // 切换暗色模式
            toggleDarkMode() {
                state.darkMode = !state.darkMode;
                document.documentElement.classList.toggle('dark', state.darkMode);
                localStorage.setItem('darkMode', state.darkMode);
                
                ToastManager.info('主题切换', state.darkMode ? '已切换到暗色模式' : '已切换到亮色模式');
            },
            
            // 获取Toast样式类
            getToastClass(type) {
                const classes = {
                    success: 'bg-green-500 text-white border-green-600',
                    error: 'bg-red-500 text-white border-red-600',
                    warning: 'bg-yellow-500 text-white border-yellow-600',
                    info: 'bg-blue-500 text-white border-blue-600'
                };
                return classes[type] || classes.info;
            },
            
            // 获取Toast图标
            getToastIcon(type) {
                const icons = {
                    success: 'fas fa-check-circle',
                    error: 'fas fa-exclamation-circle',
                    warning: 'fas fa-exclamation-triangle',
                    info: 'fas fa-info-circle'
                };
                return icons[type] || icons.info;
            },
            
            // 格式化申请状态
            formatApplicationStatus(status) {
                const statusMap = {
                    pending: { text: '处理中', class: 'bg-yellow-100 text-yellow-800', icon: 'fa-clock' },
                    approved: { text: '已批准', class: 'bg-green-100 text-green-800', icon: 'fa-check' },
                    rejected: { text: '已拒绝', class: 'bg-red-100 text-red-800', icon: 'fa-times' },
                    active: { text: '已激活', class: 'bg-blue-100 text-blue-800', icon: 'fa-database' }
                };
                return statusMap[status] || { text: '未知', class: 'bg-gray-100 text-gray-800', icon: 'fa-question' };
            }
        };
        
        // 监听器
        const watchers = {
            // 监听学号输入变化（防抖）
            watchApplyStudentId: watch(
                () => state.applyForm.studentId,
                utils.debounce((newValue) => {
                    if (newValue) {
                        methods.validateStudentId(newValue, 'apply');
                    }
                }, APP_CONFIG.DEBOUNCE_DELAY)
            ),
            
            watchQueryStudentId: watch(
                () => state.queryForm.studentId,
                utils.debounce((newValue) => {
                    if (newValue) {
                        methods.validateStudentId(newValue, 'query');
                    }
                }, APP_CONFIG.DEBOUNCE_DELAY)
            )
        };
        
        // 生命周期
        onMounted(async () => {
            // 恢复暗色模式设置
            const savedDarkMode = localStorage.getItem('darkMode') === 'true';
            if (savedDarkMode) {
                state.darkMode = true;
                document.documentElement.classList.add('dark');
            }
            
            // 初始化应用
            await methods.initializeApp();
            
            // 设置定期刷新
            setInterval(() => {
                if (state.isInitialized && !state.isLoading) {
                    methods.checkSystemHealth();
                }
            }, 30000); // 每30秒检查一次系统状态
        });
        
        // 返回组件接口
        return {
            // 状态
            state,
            toasts: ToastManager.toasts,
            
            // 计算属性
            ...computedProps,
            
            // 方法
            ...methods,
            
            // Toast管理器方法
            removeToast: ToastManager.remove,
            getToastClass: methods.getToastClass,
            getToastIcon: methods.getToastIcon
        };
    }
};

// 导出应用配置（如果需要在其他地方使用）
if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        DormDBApp,
        ToastManager,
        apiService,
        utils,
        APP_CONFIG
    };
}

// 如果在浏览器环境中，将应用挂载到全局
if (typeof window !== 'undefined') {
    window.DormDBApp = DormDBApp;
    window.ToastManager = ToastManager;
    window.apiService = apiService;
    window.utils = utils;
}
