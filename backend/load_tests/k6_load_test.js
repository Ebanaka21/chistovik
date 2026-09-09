import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Кастомные метрики
const errorRate = new Rate('errors');
const paymentLatency = new Trend('payment_latency');

// Конфигурация нагрузки
export const options = {
  stages: [
    { duration: '2m', target: 100 },  // Ramp up до 100 VUs за 2 минуты
    { duration: '5m', target: 100 },  // Держим 100 VUs 5 минут
    { duration: '2m', target: 200 },  // Ramp up до 200 VUs
    { duration: '5m', target: 200 },  // Держим 200 VUs 5 минут
    { duration: '2m', target: 500 },  // Stress test до 500 VUs
    { duration: '5m', target: 500 },  // Держим 500 VUs 5 минут
    { duration: '2m', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% запросов < 500ms
    http_req_failed: ['rate<0.01'],   // < 1% ошибок
    errors: ['rate<0.05'],            // < 5% ошибок в кастомной метрике
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8080';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || 'test_token';

// Тест 1: Health check под нагрузкой
export function testHealthCheck() {
  const res = http.get(`${BASE_URL}/health`);
  
  check(res, {
    'health status is 200': (r) => r.status === 200,
    'health response time < 100ms': (r) => r.timings.duration < 100,
  });
  
  errorRate.add(res.status !== 200);
  sleep(1);
}

// Тест 2: Аутентификация под нагрузкой
export function testAuth() {
  const payload = JSON.stringify({
    email: `user${__VU}@test.com`,
    password: 'test_password_123',
  });
  
  const params = {
    headers: { 'Content-Type': 'application/json' },
  };
  
  const res = http.post(`${BASE_URL}/api/v1/auth/login`, payload, params);
  
  check(res, {
    'auth status is 200 or 401': (r) => r.status === 200 || r.status === 401,
    'auth response time < 300ms': (r) => r.timings.duration < 300,
  });
  
  errorRate.add(res.status !== 200 && res.status !== 401);
  sleep(2);
}

// Тест 3: Создание платежа (критичный путь)
export function testPaymentCreation() {
  const payload = JSON.stringify({
    user_id: `user_${__VU}`,
    author_id: 'author_123',
    plan_id: 'plan_456',
    amount_kopecks: 29900,
  });
  
  const params = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${AUTH_TOKEN}`,
      'Idempotency-Key': `payment_${__VU}_${Date.now()}`,
    },
  };
  
  const start = Date.now();
  const res = http.post(`${BASE_URL}/api/v1/payments/create`, payload, params);
  const duration = Date.now() - start;
  
  paymentLatency.add(duration);
  
  check(res, {
    'payment status is 201': (r) => r.status === 201,
    'payment has payment_id': (r) => {
      const body = r.json();
      return body && body.payment_id;
    },
    'payment response time < 1000ms': (r) => r.timings.duration < 1000,
  });
  
  errorRate.add(res.status !== 201);
  sleep(3);
}

// Тест 4: Стриминг медиа
export function testStreaming() {
  const contentId = 'content_123';
  const res = http.get(`${BASE_URL}/stream/audio/${contentId}/master.m3u8?token=test_token`);
  
  check(res, {
    'streaming status is 200': (r) => r.status === 200,
    'streaming content-type is m3u8': (r) => 
      r.headers['Content-Type'] === 'application/vnd.apple.mpegurl',
    'streaming response time < 200ms': (r) => r.timings.duration < 200,
  });
  
  errorRate.add(res.status !== 200);
  sleep(1);
}

// Тест 5: Загрузка контента (chunked upload)
export function testChunkedUpload() {
  // Симулируем загрузку 5 чанков по 1MB
  for (let i = 0; i < 5; i++) {
    const chunk = new ArrayBuffer(1024 * 1024); // 1MB
    const res = http.post(
      `${BASE_URL}/api/v1/upload/chunk`,
      chunk,
      {
        headers: {
          'Content-Type': 'application/octet-stream',
          'Authorization': `Bearer ${AUTH_TOKEN}`,
          'X-Upload-Id': `upload_${__VU}`,
          'X-Chunk-Index': i.toString(),
        },
      }
    );
    
    check(res, {
      [`chunk ${i} status is 200`]: (r) => r.status === 200,
    });
    
    errorRate.add(res.status !== 200);
    sleep(0.5);
  }
}

// Тест 6: Job queue под нагрузкой
export function testJobQueue() {
  const payload = JSON.stringify({
    job_type: 'TranscodeVideo',
    input_path: '/tmp/test_video.mp4',
    output_dir: '/tmp/output',
    qualities: ['360p', '720p', '1080p'],
  });
  
  const params = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${AUTH_TOKEN}`,
    },
  };
  
  const res = http.post(`${BASE_URL}/api/v1/jobs`, payload, params);
  
  check(res, {
    'job created status is 201': (r) => r.status === 201,
    'job has job_id': (r) => {
      const body = r.json();
      return body && body.job_id;
    },
  });
  
  errorRate.add(res.status !== 201);
  sleep(2);
}

// Тест 7: Graceful degradation (проверка отклонения при высокой нагрузке)
export function testGracefulDegradation() {
  const res = http.get(`${BASE_URL}/api/v1/user/profile`, {
    headers: { 'Authorization': `Bearer ${AUTH_TOKEN}` },
  });
  
  // При высокой нагрузке должны получать 503
  check(res, {
    'response is 200 or 503': (r) => r.status === 200 || r.status === 503,
  });
  
  if (res.status === 503) {
    console.log(`VU ${__VU}: Graceful degradation triggered (503)`);
  }
  
  sleep(1);
}

// Основной сценарий
export default function () {
  // Выбираем случайный тест
  const testType = Math.floor(Math.random() * 7);
  
  switch (testType) {
    case 0:
      testHealthCheck();
      break;
    case 1:
      testAuth();
      break;
    case 2:
      testPaymentCreation();
      break;
    case 3:
      testStreaming();
      break;
    case 4:
      testChunkedUpload();
      break;
    case 5:
      testJobQueue();
      break;
    case 6:
      testGracefulDegradation();
      break;
  }
}

// Обработка результатов
export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'load_test_results.json': JSON.stringify(data, null, 2),
  };
}

function textSummary(data, opts) {
  return JSON.stringify(data.metrics, null, 2);
}
