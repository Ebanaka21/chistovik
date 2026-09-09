# Security Audit Recommendations — Чистовик

## Версия: 1.0
## Дата: 2026-01-XX
## Статус: Pre-Audit Preparation

---

## 📋 Содержание

1. [Обзор](#обзор)
2. [Внутренний security checklist](#внутренний-security-checklist)
3. [Рекомендации для external audit](#рекомендации-для-external-audit)
4. [Потенциальные уязвимости](#потенциальные-уязвимости)
5. [Инструменты для аудита](#инструменты-для-аудита)
6. [Timeline аудита](#timeline-аудита)

---

## Обзор

Этот документ содержит рекомендации по подготовке к внешнему security audit и список внутренних проверок безопасности.

**Цель:** Подготовить проект к независимому penetration testing через 3-6 месяцев после запуска.

---

## Внутренний security checklist

### Аутентификация и авторизация ✅

- [x] JWT tokens с валидацией expiration
- [x] Refresh token rotation
- [x] SHA-256 для хэширования токенов
- [x] Password policy validation (длина + banned passwords)
- [x] Account lockout после N неудачных попыток
- [x] Rate limiting для auth endpoints (fail-closed)
- [x] Audit logging всех аутентификаций

**Рекомендации для аудита:**
- Проверить реализацию JWT validation
- Тестировать refresh token rotation
- Проверить account lockout логику
- Audit log completeness

---

### API Protection ✅

- [x] Rate limiting через Redis (реальный, не заглушка)
- [x] Персональные лимиты по user_id
- [x] Fail-closed для auth endpoints
- [x] Извлечение IP из X-Forwarded-For
- [x] Проверка доверенных прокси
- [x] Circuit breaker для внешних API
- [x] Exponential backoff с jitter
- [x] CORS с валидацией origin
- [x] Constant-time comparison для HMAC

**Рекомендации для аудита:**
- Тестировать rate limiting под нагрузкой
- Проверить X-Forwarded-For spoofing
- Проверить CORS configuration
- Тестировать circuit breaker behavior

---

### Media Protection ✅

- [x] HLS стриминг с AES-128
- [x] Временные токены доступа (2 часа)
- [x] Forensic watermark (стеганография)
- [x] Rate limiting на стриминг
- [x] Проверка Referer (защита от хотлинкинга)
- [x] Реальное S3-хранилище через rust-s3
- [x] Job queue для FFmpeg (защита от OOM)
- [x] Chunked upload для больших файлов
- [x] Quotas и лимиты для пользователей

**Рекомендации для аудита:**
- Тестировать извлечение watermark из слитого контента
- Проверить HLS token expiration
- Тестировать hotlinking protection
- Проверить S3 bucket permissions
- Тестировать chunked upload edge cases

---

### Payment Security ✅

- [x] HMAC верификация webhook'ов от ЮKassa
- [x] Idempotency на mutation-эндпоинтах
- [x] Circuit breaker для ЮKassa API
- [x] Audit logging всех платежей
- [x] Constant-time comparison для HMAC

**Рекомендации для аудита:**
- Тестировать webhook signature verification
- Проверить idempotency при повторных запросах
- Тестировать payment flow end-to-end
- Проверить PCI DSS compliance (через ЮKassa)

---

### Data Protection ✅

- [x] Шифрование sensitive data at rest
- [x] Secure password hashing (bcrypt)
- [x] Audit logging всех критичных действий
- [x] GDPR compliance (удаление данных)
- [x] 152-ФЗ compliance (персональные данные)

**Рекомендации для аудита:**
- Проверить encryption at rest
- Тестировать data deletion flow
- Проверить PII handling
- Audit log completeness

---

### Infrastructure Security ✅

- [x] Security headers (CSP, HSTS, X-Frame-Options)
- [x] HTTPS enforcement
- [x] Graceful degradation при нагрузке
- [x] Health checks для Kubernetes
- [x] Автоматические бэкапы с шифрованием

**Рекомендации для аудита:**
- Тестировать security headers
- Проверить HTTPS configuration
- Тестировать graceful degradation
- Проверить backup encryption

---

## Рекомендации для external audit

### 1. Выбор audit компании

**Критерии выбора:**
- Опыт с Rust/Actix проектами
- Сертификация OSCP, CEH
- Опыт с payment systems
- Знание российского законодательства (152-ФЗ)

**Рекомендуемые компании:**
- Positive Technologies
- Kaspersky Lab
- BI.ZONE
- Security Vision

### 2. Scope аудита

**In Scope:**
- Backend API (все эндпоинты)
- Аутентификация и авторизация
- Payment flow
- Media streaming
- File upload/download
- Database security
- Infrastructure configuration

**Out of Scope:**
- Frontend (статические файлы)
- Third-party services (ЮKassa, S3)
- Physical security

### 3. Предоставить аудиторам

**Документация:**
- API documentation (OpenAPI spec)
- Architecture diagrams
- Data flow diagrams
- Security controls documentation
- This security audit recommendations document

**Доступ:**
- Staging environment
- Test accounts (author, fan, admin)
- Database read-only access
- Log files (last 30 days)
- Configuration files (без секретов)

**Код:**
- Full source code access
- Git history
- Dependency list (Cargo.lock)
- Migration files

### 4. Тест-кейсы для аудиторов

**Аутентификация:**
```bash
# Тест 1: Brute force protection
for i in {1..20}; do
  curl -X POST /api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{"email":"test@example.com","password":"wrong"}'
done
# Ожидаемый результат: account locked после 5 попыток

# Тест 2: JWT token manipulation
curl -X GET /api/v1/user/profile \
  -H "Authorization: Bearer expired_token"
# Ожидаемый результат: 401 Unauthorized

# Тест 3: Refresh token rotation
curl -X POST /api/v1/auth/refresh \
  -H "Content-Type: application/json" \
  -d '{"refresh_token":"old_token"}'
# Ожидаемый результат: новый token, старый отозван
```

**Rate Limiting:**
```bash
# Тест 4: Rate limit bypass attempt
for i in {1..200}; do
  curl -X GET /api/v1/user/profile \
    -H "Authorization: Bearer valid_token" &
done
wait
# Ожидаемый результат: часть запросов отклонена (429)

# Тест 5: X-Forwarded-For spoofing
curl -X GET /api/v1/user/profile \
  -H "X-Forwarded-For: 1.2.3.4" \
  -H "Authorization: Bearer valid_token"
# Ожидаемый результат: rate limit по реальному IP, не по XFF
```

**Media Protection:**
```bash
# Тест 6: HLS token expiration
curl -X GET /stream/audio/content_123/master.m3u8?token=expired_token
# Ожидаемый результат: 401 Unauthorized

# Тест 7: Hotlinking protection
curl -X GET /stream/audio/content_123/master.m3u8?token=valid_token \
  -H "Referer: http://evil.com"
# Ожидаемый результат: 403 Forbidden

# Тест 8: Watermark extraction
# Предоставить слитый файл с watermark
# Ожидаемый результат: идентификация пользователя
```

**Payment Security:**
```bash
# Тест 9: Webhook signature bypass
curl -X POST /api/v1/payments/webhook \
  -H "Content-Type: application/json" \
  -H "X-Yoo-Signature: invalid_signature" \
  -d '{"event":"payment.succeeded","object":{"id":"test"}}'
# Ожидаемый результат: 401 Unauthorized

# Тест 10: Idempotency test
curl -X POST /api/v1/payments/create \
  -H "Idempotency-Key: test_key_123" \
  -H "Authorization: Bearer valid_token" \
  -d '{"amount":100}'
# Повторить запрос с тем же ключом
# Ожидаемый результат: одинаковый payment_id
```

---

## Потенциальные уязвимости

### Высокий приоритет

1. **SQL Injection**
   - **Риск:** Использование raw SQL queries
   - **Митигация:** SQLx с parameterized queries
   - **Проверка:** Аудиторы должны проверить все SQL queries

2. **Authentication Bypass**
   - **Риск:** Ошибки в JWT validation
   - **Митигация:** Strict validation с expiration check
   - **Проверка:** Тестировать expired/malformed tokens

3. **Payment Fraud**
   - **Риск:** Manipulation payment amounts
   - **Митигация:** Server-side validation + idempotency
   - **Проверка:** Тестировать payment flow с разными amounts

### Средний приоритет

4. **Rate Limiting Bypass**
   - **Риск:** IP spoofing через X-Forwarded-For
   - **Митигация:** Trusted proxy validation
   - **Проверка:** Тестировать с разными XFF headers

5. **File Upload Vulnerabilities**
   - **Риск:** Malicious file upload
   - **Митигация:** File type validation + size limits
   - **Проверка:** Тестировать upload с malicious files

6. **Information Disclosure**
   - **Риск:** Error messages reveal internal details
   - **Митигация:** Generic error messages
   - **Проверка:** Тестировать error handling

### Низкий приоритет

7. **CORS Misconfiguration**
   - **Риск:** Overly permissive CORS
   - **Митигация:** Strict origin validation
   - **Проверка:** Тестировать CORS с разными origins

8. **Session Fixation**
   - **Риск:** Session token reuse
   - **Митигация:** Token rotation on login
   - **Проверка:** Тестировать session management

---

## Инструменты для аудита

### Автоматизированные инструменты

**Web Application:**
- OWASP ZAP (Zed Attack Proxy)
- Burp Suite Professional
- Nikto Web Scanner
- SQLMap (SQL injection testing)

**Network:**
- Nmap (port scanning)
- Nessus (vulnerability scanning)
- OpenVAS (open source vulnerability scanner)

**Code Analysis:**
- cargo-audit (Rust dependency vulnerabilities)
- clippy (Rust linter)
- SonarQube (code quality)

**Infrastructure:**
- Docker Bench Security
- CIS Benchmarks
- Lynis (Linux auditing)

### Ручное тестирование

**Authentication:**
- Manual JWT token manipulation
- Brute force attempts
- Session hijacking attempts

**Authorization:**
- Privilege escalation tests
- IDOR (Insecure Direct Object References)
- Access control testing

**Business Logic:**
- Payment flow manipulation
- Subscription bypass attempts
- Content access without subscription

---

## Timeline аудита

### Phase 1: Preparation (2 недели)

**Неделя 1:**
- [ ] Выбор audit компании
- [ ] Подготовка документации
- [ ] Настройка staging environment
- [ ] Создание тестовых аккаунтов

**Неделя 2:**
- [ ] Предоставление доступа аудиторам
- [ ] Kick-off meeting
- [ ] Определение scope и timeline
- [ ] Подписание NDA

### Phase 2: Audit Execution (3-4 недели)

**Неделя 3-4:**
- [ ] Automated scanning
- [ ] Manual penetration testing
- [ ] Code review
- [ ] Infrastructure assessment

**Неделя 5-6:**
- [ ] Vulnerability verification
- [ ] Exploitation attempts
- [ ] Business logic testing
- [ ] Report preparation

### Phase 3: Reporting & Remediation (2 недели)

**Неделя 7:**
- [ ] Final report delivery
- [ ] Findings presentation
- [ ] Remediation planning
- [ ] Priority assignment

**Неделя 8:**
- [ ] Critical fixes (высокий приоритет)
- [ ] Retest critical findings
- [ ] Medium priority fixes
- [ ] Documentation updates

### Phase 4: Validation (1 неделя)

**Неделя 9:**
- [ ] Retest all findings
- [ ] Final validation
- [ ] Compliance verification
- [ ] Sign-off

**Общая длительность:** 9 недель (~2 месяца)

---

## Бюджет аудита

**Ориентировочная стоимость:**
- Small audit (web app only): $15,000 - $25,000
- Medium audit (web + infrastructure): $30,000 - $50,000
- Comprehensive audit (full stack): $50,000 - $80,000

**Дополнительные расходы:**
- Retest after fixes: $5,000 - $10,000
- Compliance certification: $10,000 - $20,000
- Ongoing monitoring: $2,000 - $5,000/месяц

---

## Post-Audit Actions

### Немедленные действия (0-7 дней)
- [ ] Fix critical vulnerabilities
- [ ] Patch high-severity issues
- [ ] Update security documentation
- [ ] Notify stakeholders

### Краткосрочные действия (1-4 недели)
- [ ] Fix medium-severity issues
- [ ] Implement additional monitoring
- [ ] Update security policies
- [ ] Train development team

### Долгосрочные действия (1-3 месяца)
- [ ] Fix low-severity issues
- [ ] Implement security automation
- [ ] Regular security training
- [ ] Schedule next audit

---

## Compliance Requirements

### 152-ФЗ (Россия)
- [ ] Персональные данные хранятся в РФ
- [ ] Согласие на обработку данных
- [ ] Право на удаление данных
- [ ] Уведомление Роскомнадзора

### GDPR (если работаем с ЕС)
- [ ] Data Protection Officer
- [ ] Privacy Policy
- [ ] Data Processing Agreement
- [ ] Right to be forgotten

### PCI DSS (платежи)
- [ ] PCI DSS compliance через ЮKassa
- [ ] Не храним данные карт
- [ ] Secure transmission (TLS 1.2+)
- [ ] Regular security scans

---

## Контакты

**Security Team:**
- CTO: cto@chistovik.ru
- Security Lead: security@chistovik.ru
- DevOps Lead: devops@chistovik.ru

**Emergency Contacts:**
- 24/7 Security Hotline: +7 (XXX) XXX-XX-XX
- Incident Response Team: incident@chistovik.ru

---

**© 2026 Чистовик. Все права защищены.**
