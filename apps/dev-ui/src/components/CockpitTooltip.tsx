import React from 'react';

interface Props {
  header: string;
  current: string;
  sections: TooltipSection[];
  target?: string;
}

interface TooltipSection {
  title: string;
  items: TooltipItem[];
}

interface TooltipItem {
  text: string;
  type?: 'default' | 'success' | 'warning';
  indent?: boolean;
}

export const CockpitTooltip: React.FC<Props> = ({ header, current, sections, target }) => {
  return (
    <div className="cockpit-tooltip">
      <div className="tooltip-header">{header}</div>
      <div className="tooltip-current">{current}</div>
      {sections.map((section, idx) => (
        <div key={idx} className="tooltip-section">
          <div className="tooltip-section-title">{section.title}</div>
          <ul>
            {section.items.map((item, itemIdx) => (
              <li
                key={itemIdx}
                className={item.type ? `tooltip-${item.type}` : ''}
                style={item.indent ? { paddingLeft: '1.5rem' } : {}}
              >
                {item.text}
              </li>
            ))}
          </ul>
        </div>
      ))}
      {target && <div className="tooltip-target">{target}</div>}
    </div>
  );
};
