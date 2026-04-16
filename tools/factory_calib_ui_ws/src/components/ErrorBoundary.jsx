import React from 'react';

export class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, message: '' };
  }

  static getDerivedStateFromError(error) {
    return {
      hasError: true,
      message: error?.message || String(error || 'Unknown error'),
    };
  }

  componentDidCatch(error) {
    // Keep a console trace for frontend debugging.
    // eslint-disable-next-line no-console
    console.error('Factory UI crashed:', error);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="app shell">
          <section className="card glass">
            <div className="sectionTitle">
              <h2>UI Error</h2>
            </div>
            <div className="tip">A component crashed. Refresh and retry.</div>
            <pre className="box">{this.state.message}</pre>
          </section>
        </div>
      );
    }
    return this.props.children;
  }
}
