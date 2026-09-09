# Disaster Recovery Runbook — Чистовик

## Версия: 1.0
## Дата: 2026-01-XX
## Статус: Production Ready

---

## 📋 Содержание

1. [Обзор](#обзор)
2. [RTO и RPO](#rto-и-rpo)
3. [Роли и ответственность](#роли-и-ответственность)
4. [Сценарии восстановления](#сценарии-восстановления)
5. [Контакты](#контакты)
6. [Checklist восстановления](#checklist-восстановления)

---

## Обзор

Этот документ описывает процедуры восстановления сервиса Чистовик в случае катастрофических сбоев.

**Целевые показатели:**
- **RTO (Recovery Time Objective):** 1 час
- **RPO (Recovery Point Objective):** 1 час (потеря данных не более 1 часа)

**Критичные компоненты:**
1. PostgreSQL база данных
2. Redis (кэш, rate limiting, job queue)
3. S3 хранилище (медиафайлы)
4. Backend сервис (Rust/Actix)
5. Frontend (статические файлы)

---

## RTO и RPO

| Компонент | RTO | RPO | Стратегия восстановления |
|-----------|-----|-----|--------------------------|
| PostgreSQL | 30 мин | 1 час | Восстановление из бэкапа + WAL replay |
| Redis | 15 мин | 0 (пересоздаётся) | Перезапуск с пустым кэшем |
| S3 | 1 час | 0 (immutable) | Восстановление из offsite копии |
| Backend | 30 мин | 0 | Деплой из Docker image |
| Frontend | 15 мин | 0 | Деплой из CDN |

**Общий RTO:** 1 час (все компоненты параллельно)
**Общий RPO:** 1 час (определяется PostgreSQL бэкапами)

---

## Роли и ответственность

### Incident Commander (IC)
**Ответственный:** CTO / Tech Lead
**Обязанности:**
- Координация восстановления
- Коммуникация с командой
- Принятие критичных решений
- Эскалация при необходимости

### Database Administrator (DBA)
**Ответственный:** Backend Lead
**Обязанности:**
- Восстановление PostgreSQL
- Проверка целостности данных
- Replay WAL логов

### Infrastructure Engineer
**Ответственный:** DevOps Engineer
**Обязанности:**
- Восстановление Redis
- Восстановление S3
- Деплой сервисов

### Communication Lead
**Ответственный:** Product Manager
**Обязанности:**
- Коммуникация с пользователями
- Обновление status page
- Пост-мортем документация

---

## Сценарии восстановления

### Сценарий 1: Потеря PostgreSQL базы данных

**Симптомы:**
- Все запросы к API возвращают 500
- Логи показывают ошибки подключения к БД
- Health check возвращает `{"status": "unhealthy", "checks": {"database": "down"}}`

**Процедура восстановления:**

```bash
# 1. Остановить backend сервис
systemctl stop chistovik-backend

# 2. Определить последний рабочий бэкап
aws s3 ls s3://chistovik-backups/ | tail -20

# 3. Скачать бэкап
aws s3 cp s3://chistovik-backups/chistovik_backup_20260115_120000.sql.gz /tmp/

# 4. Проверить целостность бэкапа
gunzip -t /tmp/chistovik_backup_20260115_120000.sql.gz

# 5. Восстановить базу данных
createdb chistovik_restored
pg_restore -d chistovik_restored /tmp/chistovik_backup_20260115_120000.sql.gz

# 6. Проверить целостность данных
psql -d chistovik_restored -c "SELECT COUNT(*) FROM users;"
psql -d chistovik_restored -c "SELECT COUNT(*) FROM payments;"

# 7. Rename database
dropdb chistovik
psql -c "ALTER DATABASE chistovik_restored RENAME TO chistovik;"

# 8. Запустить backend сервис
systemctl start chistovik-backend

# 9. Проверить health check
curl http://localhost:8080/health

# 10. Проверить логи
journalctl -u chistovik-backend -f
```

**Время восстановления:** ~30 минут
**Потеря данных:** До 1 часа (с момента последнего бэкапа)

---

### Сценарий 2: Отказ Redis

**Симптомы:**
- Rate limiting не работает
- Job queue остановлена
- Логи показывают ошибки подключения к Redis

**Процедура восстановления:**

```bash
# 1. Проверить статус Redis
systemctl status redis

# 2. Перезапустить Redis
systemctl restart redis

# 3. Проверить подключение
redis-cli ping

# 4. Очистить кэш (если нужно)
redis-cli FLUSHALL

# 5. Проверить backend логи
journalctl -u chistovik-backend -f | grep redis

# 6. Проверить job queue
curl http://localhost:8080/api/v1/admin/jobs/stats
```

**Время восстановления:** ~15 минут
**Потеря данных:** 0 (Redis пересоздаётся автоматически)

---

### Сценарий 3: Потеря S3 хранилища

**Симптомы:**
- Медиафайлы недоступны
- Стриминг не работает
- Логи показывают ошибки S3

**Процедура восстановления:**

```bash
# 1. Проверить доступность S3
aws s3 ls s3://chistovik-media/ | head -10

# 2. Если S3 недоступен — восстановить из offsite копии
aws s3 sync s3://chistovik-backups/media/ s3://chistovik-media/

# 3. Проверить целостность файлов
aws s3 ls s3://chistovik-media/ | wc -l

# 4. Обновить CDN кэш
curl -X POST https://api.cdn-provider.com/purge \
  -H "Authorization: Bearer $CDN_TOKEN" \
  -d '{"paths": ["/*"]}'

# 5. Проверить стриминг
curl -I http://localhost:8080/stream/audio/content_123/master.m3u8
```

**Время восстановления:** ~1 час (зависит от объёма данных)
**Потеря данных:** 0 (S3 immutable)

---

### Сценарий 4: Полный отказ дата-центра

**Симптомы:**
- Все сервисы недоступны
- Нет подключения к серверам
- Мониторинг показывает downtime

**Процедура восстановления:**

```bash
# 1. Активировать DR сайт (если есть)
aws ec2 start-instances --instance-ids i-dr-site-001

# 2. Восстановить PostgreSQL из offsite бэкапа
# (см. Сценарий 1)

# 3. Восстановить Redis
# (см. Сценарий 2)

# 4. Восстановить S3
# (см. Сценарий 3)

# 5. Деплой backend
docker pull chistovik/backend:latest
docker run -d --name chistovik-backend \
  -e DATABASE_URL=postgres://... \
  -e REDIS_URL=redis://... \
  -p 8080:8080 \
  chistovik/backend:latest

# 6. Деплой frontend
aws s3 sync dist/ s3://chistovik-frontend/
aws cloudfront create-invalidation --distribution-id E123456 --paths "/*"

# 7. Обновить DNS (если нужно)
aws route53 change-resource-record-sets --hosted-zone-id Z123456 --change-batch file://dns-change.json

# 8. Проверить все сервисы
curl http://chistovik.ru/health
curl http://chistovik.ru/
```

**Время восстановления:** ~2 часа
**Потеря данных:** До 1 часа

---

## Контакты

### Primary On-Call
- **Имя:** [Имя Фамилия]
- **Телефон:** +7 (XXX) XXX-XX-XX
- **Telegram:** @username
- **Email:** oncall@chistovik.ru

### Secondary On-Call
- **Имя:** [Имя Фамилия]
- **Телефон:** +7 (XXX) XXX-XX-XX
- **Telegram:** @username
- **Email:** oncall2@chistovik.ru

### Escalation
- **CTO:** +7 (XXX) XXX-XX-XX
- **Tech Lead:** +7 (XXX) XXX-XX-XX
- **DevOps Lead:** +7 (XXX) XXX-XX-XX

### External Contacts
- **Yandex Cloud Support:** +7 (495) 974-16-56
- **ЮKassa Support:** support@yookassa.ru
- **PostgreSQL Consultant:** [Контакт]

---

## Checklist восстановления

### Перед началом восстановления
- [ ] Определить тип инцидента
- [ ] Назначить Incident Commander
- [ ] Создать incident channel в Slack/Telegram
- [ ] Уведомить stakeholders
- [ ] Обновить status page

### Во время восстановления
- [ ] Следовать процедуре из соответствующего сценария
- [ ] Логировать все действия
- [ ] Проверять целостность данных после каждого шага
- [ ] Тестировать сервис после восстановления
- [ ] Обновлять status page

### После восстановления
- [ ] Проверить все эндпоинты
- [ ] Проверить метрики и логи
- [ ] Уведомить пользователей о восстановлении
- [ ] Закрыть incident channel
- [ ] Написать пост-мортем (в течение 48 часов)
- [ ] Обновить runbook если нужно

---

## Пост-мортем шаблон

### Incident Summary
- **Дата:** [Дата]
- **Время начала:** [Время]
- **Время окончания:** [Время]
- **Длительность:** [X часов/минут]
- **Severity:** [P1/P2/P3]

### Impact
- **Затронутые пользователи:** [Количество]
- **Потерянные транзакции:** [Количество]
- **Финансовые потери:** [Сумма]

### Root Cause
[Описание корневой причины]

### Timeline
- **[Время]:** [Событие]
- **[Время]:** [Событие]

### What Went Well
- [Пункт 1]
- [Пункт 2]

### What Went Wrong
- [Пункт 1]
- [Пункт 2]

### Action Items
- [ ] [Задача] — [Ответственный] — [Дедлайн]
- [ ] [Задача] — [Ответственный] — [Дедлайн]

---

## Тестирование DR плана

### Ежеквартальные DR drills
1. **Q1:** Восстановление PostgreSQL из бэкапа
2. **Q2:** Восстановление S3 из offsite копии
3. **Q3:** Полный failover на DR сайт
4. **Q4:** Tabletop exercise (симуляция инцидента)

### Метрики успешности
- RTO достигнут: ✅ / ❌
- RPO достигнут: ✅ / ❌
- Все данные восстановлены: ✅ / ❌
- Сервис работает корректно: ✅ / ❌

---

## Приложение A: Полезные команды

```bash
# Проверка бэкапов
aws s3 ls s3://chistovik-backups/ --recursive | grep .sql.gz | tail -10

# Проверка целостности бэкапа
gunzip -t /path/to/backup.sql.gz

# Восстановление PostgreSQL
pg_restore -d chistovik /path/to/backup.sql.gz

# Проверка Redis
redis-cli INFO server
redis-cli INFO memory

# Проверка S3
aws s3 ls s3://chistovik-media/ | wc -l

# Проверка backend
curl http://localhost:8080/health
curl http://localhost:8080/health/metrics

# Проверка job queue
curl http://localhost:8080/api/v1/admin/jobs/stats

# Просмотр логов
journalctl -u chistovik-backend -f
docker logs chistovik-backend

# Проверка метрик
curl http://localhost:8080/health/metrics | grep -E "http_requests_total|db_connections"
```

---

## Приложение B: Диаграмма восстановления

```
┌─────────────────────────────────────────────────────────────┐
│                    INCIDENT DETECTED                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              1. Notify Incident Commander                    │
│              2. Create incident channel                      │
│              3. Update status page                           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Identify Failure Type                           │
│              - Database? → Scenario 1                        │
│              - Redis? → Scenario 2                           │
│              - S3? → Scenario 3                              │
│              - Full DC? → Scenario 4                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Execute Recovery Procedure                      │
│              - Follow runbook                                │
│              - Log all actions                               │
│              - Test after each step                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Verify Recovery                                 │
│              - Health checks                                 │
│              - Data integrity                                │
│              - End-to-end tests                              │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Post-Incident                                   │
│              - Notify users                                  │
│              - Write post-mortem                             │
│              - Update runbook                                │
└─────────────────────────────────────────────────────────────┘
```

---

**© 2026 Чистовик. Все права защищены.**
