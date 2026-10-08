import { render } from 'solid-js/web';
import App from './App';
import './styles.css';
import './router';

render(() => <App />, document.getElementById('app')!);
