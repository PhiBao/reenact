import { Link, NavLink, Route, Routes } from 'react-router-dom';
import { Home } from './pages/Home';
import { CasePage } from './pages/CasePage';
import { Benchmark } from './pages/Benchmark';

function Mark() {
  return (
    <svg className="mark" viewBox="0 0 64 64" aria-hidden="true">
      <g fill="none" stroke="currentColor" strokeWidth="6" strokeLinecap="square">
        <path d="M12 26V12h14" />
        <path d="M38 12h14v14" />
        <path d="M52 38v14H38" />
        <path d="M26 52H12V38" />
      </g>
      <circle cx="32" cy="32" r="9" fill="var(--red)" />
    </svg>
  );
}

export function App() {
  return (
    <>
      <header className="top">
        <Link to="/" className="brand" aria-label="Reenact home">
          <Mark />
          <span>Reenact</span>
        </Link>
        <nav aria-label="Primary">
          <NavLink to="/">Cases</NavLink>
          <NavLink to="/benchmark">Benchmark</NavLink>
          <a href="https://github.com/PhiBao/reenact">Source</a>
        </nav>
      </header>
      <main>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/c/:id" element={<CasePage />} />
          <Route path="/benchmark" element={<Benchmark />} />
          <Route path="*" element={<p className="empty">Nothing filed under that name.</p>} />
        </Routes>
      </main>
      <footer className="foot">
        <span>Reproductions by Kane CLI. Every verdict links to its evidence.</span>
      </footer>
    </>
  );
}
