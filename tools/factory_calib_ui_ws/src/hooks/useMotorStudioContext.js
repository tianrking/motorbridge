import React from 'react';

const MotorStudioContext = React.createContext(null);

export function MotorStudioProvider({ value, children }) {
  return React.createElement(MotorStudioContext.Provider, { value }, children);
}

export function useMotorStudioContext() {
  const value = React.useContext(MotorStudioContext);
  if (!value) {
    throw new Error('useMotorStudioContext must be used inside MotorStudioProvider');
  }
  return value;
}
