import { useState } from 'react';
import { Link } from 'react-router-dom';
import { Music, Video, Shield, Zap, Menu, X } from 'lucide-react';

export default function HomePage() {
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <div className="min-h-screen bg-dark">
      {/* Header */}
      <header className="fixed top-0 w-full z-50 glass-card border-b border-dark-border">
        <div className="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
          <Link to="/" className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
              <Music className="w-5 h-5 text-white" />
            </div>
            <span className="text-xl font-bold gradient-text">Чистовик</span>
          </Link>
          <nav className="hidden md:flex items-center gap-6">
            <a href="#features" className="text-gray-300 hover:text-white transition">Возможности</a>
            <a href="#pricing" className="text-gray-300 hover:text-white transition">Тарифы</a>
            <a href="#authors" className="text-gray-300 hover:text-white transition">Авторы</a>
            <Link to="/login" className="px-4 py-2 rounded-lg bg-primary hover:bg-primary-dark text-white transition">
              Войти
            </Link>
          </nav>
          <button className="md:hidden text-white" onClick={() => setMenuOpen(!menuOpen)}>
            {menuOpen ? <X /> : <Menu />}
          </button>
        </div>
        {menuOpen && (
          <div className="md:hidden px-4 pb-4 flex flex-col gap-3">
            <a href="#features" className="text-gray-300">Возможности</a>
            <a href="#pricing" className="text-gray-300">Тарифы</a>
            <a href="#authors" className="text-gray-300">Авторы</a>
            <Link to="/login" className="px-4 py-2 rounded-lg bg-primary text-white text-center">Войти</Link>
          </div>
        )}
      </header>

      {/* Hero Section */}
      <section className="pt-32 pb-20 px-4 relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-primary/10 via-transparent to-transparent" />
        <div className="absolute top-20 left-1/4 w-96 h-96 bg-primary/20 rounded-full blur-3xl" />
        <div className="absolute top-40 right-1/4 w-72 h-72 bg-accent/10 rounded-full blur-3xl" />
        
        <div className="max-w-5xl mx-auto text-center relative z-10">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full glass-card mb-8">
            <Zap className="w-4 h-4 text-accent" />
            <span className="text-sm text-gray-300">Lossless качество • Защита контента • Модульные подписки</span>
          </div>
          
          <h1 className="text-5xl md:text-7xl font-bold mb-6 leading-tight">
            Монетизируй свой контент
            <br />
            <span className="gradient-text">без посредников</span>
          </h1>
          
          <p className="text-xl text-gray-400 mb-10 max-w-3xl mx-auto">
            Создай витрину для продажи эксклюзивной музыки и видео по подписке. 
            Lossless аудио, защищённое видео, гибкие тарифы — всё в одном месте.
          </p>
          
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Link to="/author/dashboard" className="px-8 py-4 rounded-xl bg-primary hover:bg-primary-dark text-white font-semibold text-lg transition glow-purple">
              Стать автором
            </Link>
            <Link to="/aleksei-morozov" className="px-8 py-4 rounded-xl glass-card hover:border-primary-light text-white font-semibold text-lg transition">
              Смотреть витрину
            </Link>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section id="features" className="py-20 px-4">
        <div className="max-w-7xl mx-auto">
          <h2 className="text-4xl font-bold text-center mb-4">Всё для авторов контента</h2>
          <p className="text-gray-400 text-center mb-16 text-lg">Инструменты, которые помогают зарабатывать на творчестве</p>
          
          <div className="grid md:grid-cols-3 gap-8">
            <div className="glass-card rounded-2xl p-8 hover:border-primary-light transition">
              <div className="w-14 h-14 rounded-xl bg-primary/20 flex items-center justify-center mb-6">
                <Music className="w-7 h-7 text-primary-light" />
              </div>
              <h3 className="text-xl font-bold mb-3">Lossless Аудио</h3>
              <p className="text-gray-400">
                FLAC, WAV, MP3 320 — ваши треки в максимальном качестве. 
                Тизеры 15 секунд для привлечения подписчиков.
              </p>
            </div>
            
            <div className="glass-card rounded-2xl p-8 hover:border-primary-light transition">
              <div className="w-14 h-14 rounded-xl bg-accent/20 flex items-center justify-center mb-6">
                <Video className="w-7 h-7 text-accent" />
              </div>
              <h3 className="text-xl font-bold mb-3">Защищённое видео</h3>
              <p className="text-gray-400">
                HLS-стриминг с шифрованием, водяные знаки, защита от скачивания. 
                Тизеры до 60 секунд.
              </p>
            </div>
            
            <div className="glass-card rounded-2xl p-8 hover:border-primary-light transition">
              <div className="w-14 h-14 rounded-xl bg-green-500/20 flex items-center justify-center mb-6">
                <Shield className="w-7 h-7 text-green-400" />
              </div>
              <h3 className="text-xl font-bold mb-3">DRM-защита</h3>
              <p className="text-gray-400">
                Токены доступа, ограничение сессий, rate limiting. 
                Ваш контент под надёжной защитой.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Pricing Section */}
      <section id="pricing" className="py-20 px-4">
        <div className="max-w-5xl mx-auto">
          <h2 className="text-4xl font-bold text-center mb-4">Модульные подписки</h2>
          <p className="text-gray-400 text-center mb-16 text-lg">Автор сам решает, что продавать и по какой цене</p>
          
          <div className="grid md:grid-cols-3 gap-6">
            <div className="glass-card rounded-2xl p-8 text-center">
              <div className="text-3xl mb-4">🎵</div>
              <h3 className="text-xl font-bold mb-2">Музыка</h3>
              <p className="text-gray-400 mb-4">Доступ к закрытым трекам в Lossless</p>
              <div className="text-3xl font-bold text-primary-light mb-6">от 149 ₽<span className="text-sm text-gray-400">/мес</span></div>
              <ul className="text-left text-gray-300 space-y-2 mb-6">
                <li className="flex items-center gap-2">✓ FLAC / WAV / MP3 320</li>
                <li className="flex items-center gap-2">✓ Тизеры 15 сек</li>
                <li className="flex items-center gap-2">✓ Потоковое воспроизведение</li>
              </ul>
            </div>
            
            <div className="glass-card rounded-2xl p-8 text-center border-primary-light glow-purple relative">
              <div className="absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-1 bg-primary rounded-full text-xs font-bold">Популярный</div>
              <div className="text-3xl mb-4">🎬</div>
              <h3 className="text-xl font-bold mb-2">Видео</h3>
              <p className="text-gray-400 mb-4">Эксклюзивные видео в HD/4K</p>
              <div className="text-3xl font-bold text-accent mb-6">от 199 ₽<span className="text-sm text-gray-400">/мес</span></div>
              <ul className="text-left text-gray-300 space-y-2 mb-6">
                <li className="flex items-center gap-2">✓ HLS-стриминг</li>
                <li className="flex items-center gap-2">✓ Тизеры 60 сек</li>
                <li className="flex items-center gap-2">✓ Водяные знаки</li>
              </ul>
            </div>
            
            <div className="glass-card rounded-2xl p-8 text-center">
              <div className="text-3xl mb-4">🎁</div>
              <h3 className="text-xl font-bold mb-2">Комбо</h3>
              <p className="text-gray-400 mb-4">Музыка + Видео со скидкой</p>
              <div className="text-3xl font-bold text-green-400 mb-6">от 299 ₽<span className="text-sm text-gray-400">/мес</span></div>
              <ul className="text-left text-gray-300 space-y-2 mb-6">
                <li className="flex items-center gap-2">✓ Всё из «Музыка»</li>
                <li className="flex items-center gap-2">✓ Всё из «Видео»</li>
                <li className="flex items-center gap-2">✓ Экономия до 15%</li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      {/* Authors Preview */}
      <section id="authors" className="py-20 px-4">
        <div className="max-w-7xl mx-auto">
          <h2 className="text-4xl font-bold text-center mb-16">Авторы на платформе</h2>
          <div className="grid md:grid-cols-4 gap-6">
            {[
              { name: 'Алексей Морозов', type: 'Электронная музыка', avatar: '🎹' },
              { name: 'Мария Светлова', type: 'Видео-арт', avatar: '🎨' },
              { name: 'Дмитрий Волков', type: 'Инди-рок', avatar: '🎸' },
              { name: 'Анна Козлова', type: 'Мастер-классы', avatar: '📚' },
            ].map((author, i) => (
              <Link to="/aleksei-morozov" key={i} className="glass-card rounded-2xl p-6 text-center hover:border-primary-light transition cursor-pointer">
                <div className="text-5xl mb-4">{author.avatar}</div>
                <h3 className="font-bold mb-1">{author.name}</h3>
                <p className="text-sm text-gray-400">{author.type}</p>
              </Link>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 px-4">
        <div className="max-w-4xl mx-auto glass-card rounded-3xl p-12 text-center relative overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-r from-primary/20 to-accent/20" />
          <div className="relative z-10">
            <h2 className="text-4xl font-bold mb-4">Готовы начать?</h2>
            <p className="text-gray-300 mb-8 text-lg">Создайте витрину за 5 минут и начните зарабатывать на своём контенте</p>
            <Link to="/login" className="inline-block px-8 py-4 rounded-xl bg-primary hover:bg-primary-dark text-white font-semibold text-lg transition glow-purple">
              Создать витрину бесплатно
            </Link>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t border-dark-border py-12 px-4">
        <div className="max-w-7xl mx-auto grid md:grid-cols-4 gap-8">
          <div>
            <div className="flex items-center gap-2 mb-4">
              <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
                <Music className="w-5 h-5 text-white" />
              </div>
              <span className="text-xl font-bold">Чистовик</span>
            </div>
            <p className="text-gray-400 text-sm">Платформа закрытого контента для авторов и их аудитории</p>
          </div>
          <div>
            <h4 className="font-bold mb-3">Платформа</h4>
            <ul className="space-y-2 text-gray-400 text-sm">
              <li><a href="#" className="hover:text-white transition">О нас</a></li>
              <li><a href="#" className="hover:text-white transition">Блог</a></li>
              <li><a href="#" className="hover:text-white transition">Контакты</a></li>
            </ul>
          </div>
          <div>
            <h4 className="font-bold mb-3">Авторам</h4>
            <ul className="space-y-2 text-gray-400 text-sm">
              <li><a href="#" className="hover:text-white transition">Как начать</a></li>
              <li><a href="#" className="hover:text-white transition">Тарифы</a></li>
              <li><a href="#" className="hover:text-white transition">FAQ</a></li>
            </ul>
          </div>
          <div>
            <h4 className="font-bold mb-3">Правовая информация</h4>
            <ul className="space-y-2 text-gray-400 text-sm">
              <li><a href="#" className="hover:text-white transition">Оферта</a></li>
              <li><a href="#" className="hover:text-white transition">Политика конфиденциальности</a></li>
              <li><a href="#" className="hover:text-white transition">152-ФЗ</a></li>
            </ul>
          </div>
        </div>
        <div className="max-w-7xl mx-auto mt-8 pt-8 border-t border-dark-border text-center text-gray-500 text-sm">
          © 2026 Чистовик. Все права защищены.
        </div>
      </footer>
    </div>
  );
}
