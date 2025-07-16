#!/bin/bash

# DormDB API 功能测试脚本
# 测试所有API接口的功能和安全性

set -e

# 配置
BASE_URL="http://localhost:3000"
TEST_STUDENT_ID="2023010101"
ADMIN_PASSWORD="YourStrongPassword123!"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 函数：打印标题
print_title() {
    echo -e "\n${BLUE}=== $1 ===${NC}"
}

# 函数：打印成功消息
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# 函数：打印错误消息
print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 函数：打印警告消息
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# 函数：打印信息消息
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# 函数：等待用户输入
wait_for_input() {
    echo -e "${YELLOW}按回车继续...${NC}"
    read -r
}

# 函数：发送HTTP请求并解析响应
send_request() {
    local method=$1
    local url=$2
    local data=$3
    local headers=$4
    
    if [ -n "$headers" ]; then
        if [ -n "$data" ]; then
            curl -s -X "$method" "$url" -H "Content-Type: application/json" -H "$headers" -d "$data"
        else
            curl -s -X "$method" "$url" -H "$headers"
        fi
    else
        if [ -n "$data" ]; then
            curl -s -X "$method" "$url" -H "Content-Type: application/json" -d "$data"
        else
            curl -s -X "$method" "$url"
        fi
    fi
}

# 函数：提取JWT令牌
extract_jwt_token() {
    local response=$1
    echo "$response" | grep -o '"token":"[^"]*"' | sed 's/"token":"//g' | sed 's/"//g'
}

# 全局变量
JWT_TOKEN=""

echo "🔗 DormDB API 功能测试脚本"
echo "============================="
echo "基础URL: $BASE_URL"
echo "测试学号: $TEST_STUDENT_ID"
echo ""

# 1. 健康检查
print_title "1. 健康检查"
response=$(send_request "GET" "$BASE_URL/api/v1/health")
if echo "$response" | grep -q '"code":0'; then
    print_success "健康检查通过"
else
    print_error "健康检查失败"
    echo "响应: $response"
fi

# 2. 学号申请测试（预期失败，因为没有学号白名单）
print_title "2. 学号申请测试（预期失败）"
response=$(send_request "POST" "$BASE_URL/api/v1/apply" '{"identity_key":"'$TEST_STUDENT_ID'"}')
if echo "$response" | grep -q '"code":0'; then
    print_warning "学号申请意外成功，可能存在安全问题"
    echo "响应: $response"
else
    print_success "学号申请正确失败（需要先添加到白名单）"
fi

# 3. 管理员登录测试
print_title "3. 管理员登录测试"
print_info "尝试使用弱密码（应该失败）"
response=$(send_request "POST" "$BASE_URL/api/v1/admin/login" '{"password":"123456"}')
if echo "$response" | grep -q '"code":0'; then
    print_error "弱密码登录成功，存在安全风险！"
else
    print_success "弱密码登录正确被拒绝"
fi

print_info "尝试使用强密码登录"
response=$(send_request "POST" "$BASE_URL/api/v1/admin/login" '{"password":"'$ADMIN_PASSWORD'"}')
if echo "$response" | grep -q '"code":0'; then
    JWT_TOKEN=$(extract_jwt_token "$response")
    if [ -n "$JWT_TOKEN" ]; then
        print_success "管理员登录成功，获得JWT令牌"
        print_info "JWT令牌: ${JWT_TOKEN:0:50}..."
    else
        print_error "登录成功但未获得JWT令牌"
    fi
else
    print_error "管理员登录失败"
    echo "响应: $response"
    echo "请确保已设置正确的ADMIN_PASSWORD_HASH环境变量"
    exit 1
fi

# 4. 测试未认证的管理员接口访问
print_title "4. 测试未认证的管理员接口访问"
response=$(send_request "GET" "$BASE_URL/api/v1/admin/status")
if echo "$response" | grep -q '"code":0'; then
    print_error "未认证访问管理员接口成功，存在安全风险！"
else
    print_success "未认证访问管理员接口正确被拒绝"
fi

# 5. 测试认证的管理员接口访问
print_title "5. 测试认证的管理员接口访问"
if [ -n "$JWT_TOKEN" ]; then
    response=$(send_request "GET" "$BASE_URL/api/v1/admin/status" "" "Authorization: Bearer $JWT_TOKEN")
    if echo "$response" | grep -q '"code":0'; then
        print_success "认证的管理员接口访问成功"
        echo "系统状态: $(echo "$response" | grep -o '"database_status":"[^"]*"' | sed 's/"database_status":"//g' | sed 's/"//g')"
    else
        print_error "认证的管理员接口访问失败"
        echo "响应: $response"
    fi
else
    print_error "无JWT令牌，跳过认证测试"
fi

# 6. 学号管理测试
print_title "6. 学号管理测试"
if [ -n "$JWT_TOKEN" ]; then
    # 添加测试学号到白名单
    print_info "添加测试学号到白名单"
    response=$(send_request "POST" "$BASE_URL/api/v1/admin/student-ids" \
        '{"student_id":"'$TEST_STUDENT_ID'","student_name":"测试学生","class_info":"测试班级"}' \
        "Authorization: Bearer $JWT_TOKEN")
    
    if echo "$response" | grep -q '"code":0'; then
        print_success "学号添加到白名单成功"
        
        # 获取学号列表
        print_info "获取学号列表"
        response=$(send_request "GET" "$BASE_URL/api/v1/admin/student-ids?limit=10&offset=0" "" "Authorization: Bearer $JWT_TOKEN")
        if echo "$response" | grep -q '"code":0'; then
            print_success "学号列表获取成功"
            count=$(echo "$response" | grep -o '"student_id"' | wc -l)
            print_info "当前白名单中有 $count 个学号"
        else
            print_error "学号列表获取失败"
        fi
        
        # 获取学号统计
        print_info "获取学号统计"
        response=$(send_request "GET" "$BASE_URL/api/v1/admin/student-ids/stats" "" "Authorization: Bearer $JWT_TOKEN")
        if echo "$response" | grep -q '"code":0'; then
            print_success "学号统计获取成功"
            total=$(echo "$response" | grep -o '"total_count":[0-9]*' | sed 's/"total_count"://g')
            applied=$(echo "$response" | grep -o '"applied_count":[0-9]*' | sed 's/"applied_count"://g')
            print_info "总学号数: $total, 已申请: $applied"
        else
            print_error "学号统计获取失败"
        fi
        
    else
        print_error "学号添加到白名单失败"
        echo "响应: $response"
    fi
else
    print_error "无JWT令牌，跳过学号管理测试"
fi

# 7. 重新测试学号申请（应该成功）
print_title "7. 重新测试学号申请（应该成功）"
response=$(send_request "POST" "$BASE_URL/api/v1/apply" '{"identity_key":"'$TEST_STUDENT_ID'"}')
if echo "$response" | grep -q '"code":0'; then
    print_success "学号申请成功！"
    
    # 解析数据库连接信息
    db_name=$(echo "$response" | grep -o '"db_name":"[^"]*"' | sed 's/"db_name":"//g' | sed 's/"//g')
    username=$(echo "$response" | grep -o '"username":"[^"]*"' | sed 's/"username":"//g' | sed 's/"//g')
    password=$(echo "$response" | grep -o '"password":"[^"]*"' | sed 's/"password":"//g' | sed 's/"//g')
    
    print_info "数据库名: $db_name"
    print_info "用户名: $username"
    print_info "密码: ${password:0:10}..."
    
    # 测试重复申请（应该失败）
    print_info "测试重复申请（应该失败）"
    response=$(send_request "POST" "$BASE_URL/api/v1/apply" '{"identity_key":"'$TEST_STUDENT_ID'"}')
    if echo "$response" | grep -q '"code":40901'; then
        print_success "重复申请正确被拒绝"
    else
        print_error "重复申请意外成功或其他错误"
        echo "响应: $response"
    fi
    
else
    print_error "学号申请失败"
    echo "响应: $response"
fi

# 8. 申请统计测试
print_title "8. 申请统计测试"
if [ -n "$JWT_TOKEN" ]; then
    response=$(send_request "GET" "$BASE_URL/api/v1/admin/stats" "" "Authorization: Bearer $JWT_TOKEN")
    if echo "$response" | grep -q '"code":0'; then
        print_success "申请统计获取成功"
        total=$(echo "$response" | grep -o '"total_count":[0-9]*' | sed 's/"total_count"://g')
        today=$(echo "$response" | grep -o '"today_count":[0-9]*' | sed 's/"today_count"://g')
        successful=$(echo "$response" | grep -o '"successful_count":[0-9]*' | sed 's/"successful_count"://g')
        print_info "总申请数: $total, 今日申请: $today, 成功申请: $successful"
    else
        print_error "申请统计获取失败"
    fi
else
    print_error "无JWT令牌，跳过申请统计测试"
fi

# 9. 用户管理测试
print_title "9. 用户管理测试"
if [ -n "$JWT_TOKEN" ]; then
    # 获取所有用户
    print_info "获取所有用户列表"
    response=$(send_request "GET" "$BASE_URL/api/v1/admin/users" "" "Authorization: Bearer $JWT_TOKEN")
    if echo "$response" | grep -q '"code":0'; then
        print_success "用户列表获取成功"
        count=$(echo "$response" | grep -o '"identity_key"' | wc -l)
        print_info "当前系统中有 $count 个用户"
    else
        print_error "用户列表获取失败"
    fi
else
    print_error "无JWT令牌，跳过用户管理测试"
fi

# 10. 数据一致性检查
print_title "10. 数据一致性检查"
if [ -n "$JWT_TOKEN" ]; then
    print_info "执行数据一致性检查"
    response=$(send_request "POST" "$BASE_URL/api/v1/admin/repair" "" "Authorization: Bearer $JWT_TOKEN")
    if echo "$response" | grep -q '"code":0'; then
        print_success "数据一致性检查完成"
        # 提取检查结果的关键信息
        if echo "$response" | grep -q "总记录数"; then
            print_info "检查结果已生成，请查看管理员界面获取详细信息"
        fi
    else
        print_error "数据一致性检查失败"
    fi
else
    print_error "无JWT令牌，跳过数据一致性检查"
fi

# 11. 公开接口测试
print_title "11. 公开接口测试"
print_info "获取公开申请记录"
response=$(send_request "GET" "$BASE_URL/api/v1/public/applications")
if echo "$response" | grep -q '"code":0'; then
    print_success "公开申请记录获取成功"
    count=$(echo "$response" | grep -o '"identity_key_masked"' | wc -l)
    print_info "公开记录数: $count"
else
    print_error "公开申请记录获取失败"
fi

# 12. 安全测试
print_title "12. 安全测试"

# SQL注入测试
print_info "SQL注入测试"
response=$(send_request "POST" "$BASE_URL/api/v1/apply" '{"identity_key":"2023010101\""; DROP TABLE users; --"}')
if echo "$response" | grep -q '"code":0'; then
    print_error "SQL注入测试：系统可能存在SQL注入漏洞！"
else
    print_success "SQL注入测试：系统正确拒绝恶意输入"
fi

# XSS测试
print_info "XSS测试"
response=$(send_request "POST" "$BASE_URL/api/v1/apply" '{"identity_key":"<script>alert(1)</script>"}')
if echo "$response" | grep -q '"code":0'; then
    print_error "XSS测试：系统可能存在XSS漏洞！"
else
    print_success "XSS测试：系统正确拒绝恶意输入"
fi

# 13. 性能测试
print_title "13. 性能测试"
print_info "并发请求测试（10个并发健康检查）"
start_time=$(date +%s%N)
for i in {1..10}; do
    send_request "GET" "$BASE_URL/api/v1/health" &
done
wait
end_time=$(date +%s%N)
duration=$((($end_time - $start_time) / 1000000))
print_info "10个并发请求完成，耗时: ${duration}ms"

# 14. 清理测试数据
print_title "14. 清理测试数据"
if [ -n "$JWT_TOKEN" ]; then
    print_info "删除测试用户"
    response=$(send_request "DELETE" "$BASE_URL/api/v1/admin/users/$TEST_STUDENT_ID" \
        '{"reason":"测试完成，清理数据"}' \
        "Authorization: Bearer $JWT_TOKEN")
    
    if echo "$response" | grep -q '"code":0'; then
        print_success "测试用户删除成功"
    else
        print_warning "测试用户删除失败或用户不存在"
    fi
else
    print_warning "无JWT令牌，无法清理测试数据"
fi

# 测试总结
print_title "测试总结"
echo "🎯 测试完成！"
echo ""
echo "✅ 完成的测试项目："
echo "   - 健康检查"
echo "   - 学号申请流程"
echo "   - 管理员认证"
echo "   - 权限验证"
echo "   - 学号管理"
echo "   - 用户管理"
echo "   - 数据一致性检查"
echo "   - 公开接口"
echo "   - 安全测试"
echo "   - 性能测试"
echo ""
echo "🔧 访问地址："
echo "   - API文档: $BASE_URL/swagger-ui/"
echo "   - 自定义API文档: $BASE_URL/api-docs.html"
echo "   - 管理员界面: $BASE_URL/admin.html"
echo "   - 演示页面: $BASE_URL/demo.html"
echo ""
echo "📚 更多信息："
echo "   - 用户手册: ./用户手册.md"
echo "   - 安全修复报告: ./SECURITY_FIXES.md"
echo "   - 部署配置: ./.env.example.secure"
echo ""
print_success "DormDB API测试完成！"