import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Music, Mail, Phone, Eye, EyeOff } from 'lucide-react';

export default function LoginPage() {
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [role, setRole] = useState<'fan' | 'author'>('fan');
  const [showPassword, setShowPassword] = useState(false);

  return (
    <div className="min-h-screen bg-dark flex items-center justify-center px-4">
      <div className="absolute inset-0 bg-gradient-to-b from-primary/10 via-transparent to-transparent" />
      <div className="absolute top-1/4 left-1/3 w-96 h-96 bg-primary/10 rounded-full blur-3xl" />
      
      <div className="glass-card rounded-2xl p-8 max-w-md w-full relative z-10">
        <Link to="/" className="flex items-center gap-2 justify-center mb-8">
          <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
            <Music className="w-5 h-5 text-white" />
          </div>
          <span className="text-xl font-bold gradient-text">Чистовик</span>
        </Link>

        <h2 className="text-2xl font-bold text-center mb-6">
          {mode === 'login' ? 'Вход в аккаунт' : 'Регистрация'}
        </h2>

        {mode === 'register' && (
          <div className="flex gap-2 mb-6">
            <button
              onClick={() => setRole('fan')}
              className={`flex-1 py-2.5 rounded-lg text-sm font-medium transition ${role === 'fan' ? 'bg-primary text-white' : 'glass-card text-gray-300'}`}
            >
              🎧 Слушатель
            </button>
            <button
              onClick={() => setRole('author')}
              className={`flex-1 py-2.5 rounded-lg text-sm font-medium transition ${role === 'author' ? 'bg-primary text-white' : 'glass-card text-gray-300'}`}
            >
              🎵 Автор
            </button>
          </div>
        )}

        <form className="space-y-4" onSubmit={e => { e.preventDefault(); window.location.href = role === 'author' ? '/author/dashboard' : role === 'fan' && mode === 'login' ? '/user/dashboard' : '/'; }}>
          {mode === 'register' && (
            <div>
              <label className="text-sm text-gray-400 mb-1 block">Имя</label>
              <input
                type="text"
                placeholder="Ваше имя"
                className="w-full px-4 py-3 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white"
              />
            </div>
          )}
          
          <div>
            <label className="text-sm text-gray-400 mb-1 block">Email или телефон</label>
            <div className="relative">
              <input
                type="text"
                placeholder="email@example.com"
                className="w-full px-4 py-3 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white"
              />
              <Mail className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            </div>
          </div>

          <div>
            <label className="text-sm text-gray-400 mb-1 block">Пароль</label>
            <div className="relative">
              <input
                type={showPassword ? 'text' : 'password'}
                placeholder="••••••••"
                className="w-full px-4 py-3 rounded-lg bg-dark border border-dark-border focus:border-primary-light outline-none transition text-white"
              />
              <button type="button" onClick={() => setShowPassword(!showPassword)} className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300">
                {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
              </button>
            </div>
          </div>

          {mode === 'login' && (
            <div className="flex items-center justify-between text-sm">
              <label className="flex items-center gap-2 text-gray-400">
                <input type="checkbox" className="rounded border-dark-border" />
                Запомнить
              </label>
              <a href="#" className="text-primary-light hover:underline">Забыли пароль?</a>
            </div>
          )}

          <button
            type="submit"
            className="w-full py-3 rounded-lg bg-primary hover:bg-primary-dark text-white font-semibold transition glow-purple"
          >
            {mode === 'login' ? 'Войти' : 'Создать аккаунт'}
          </button>
        </form>

        <div className="mt-6 text-center text-sm text-gray-400">
          {mode === 'login' ? (
            <>Нет аккаунта? <button onClick={() => setMode('register')} className="text-primary-light hover:underline">Зарегистрироваться</button></>
          ) : (
            <>Уже есть аккаунт? <button onClick={() => setMode('login')} className="text-primary-light hover:underline">Войти</button></>
          )}
        </div>

        <div className="mt-6 pt-6 border-t border-dark-border">
          <p className="text-center text-sm text-gray-400 mb-3">Или войти через</p>
          <div className="flex gap-3">
            <button className="flex-1 py-2.5 rounded-lg glass-card hover:border-primary-light transition text-sm">
              📱 Telegram
            </button>
            <button className="flex-1 py-2.5 rounded-lg glass-card hover:border-primary-light transition text-sm">
              💬 VK ID
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
