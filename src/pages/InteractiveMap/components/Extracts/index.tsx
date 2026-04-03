import React from 'react';
import { Group, Text } from 'react-konva';

import {
  getExtractDisplayName,
  getIconCDN,
  mouseClickEvent,
  mouseHoverEvent,
} from '@/pages/InteractiveMap/utils';

import Image from '../Image';

interface ExtractsProps {
  extracts: InteractiveMap.Extract[];
  show: InteractiveMap.Faction[];
}

const extractColorMap: Record<string, string> = {
  pmc: '#88ff00',
  scav: '#ff8800',
  shared: '#00ccff',
};

const Index = (props: ExtractsProps & InteractiveMap.UtilProps) => {
  const {
    extracts = [],
    baseMapStatus,
    mapScale,
    activeLayer,
    heightRange,
    real2imagePos,
    show,
    iconScale = 1,
    textScale = 1,
    minIconSize = 0,
    minFontSize = 0,
  } = props;
  if (baseMapStatus === 'loaded' && show.length > 0) {
    return (
      <Group>
        {extracts.map((extract) => {
          const extractHeight = extract.position.y;
          let active = true;
          if (activeLayer) {
            if (extractHeight < heightRange[0] || extractHeight > heightRange[1]) {
              active = false;
            }
          }
          if (show.includes(extract.faction)) {
            const displayName = getExtractDisplayName(extract.id, extract.name);
            return (
              <Group
                key={`im-extract-group-${extract.id}-${extract.position.x}-${extract.position.z}`}
                id={`im-extract-group-${extract.id}-${extract.position.x}-${extract.position.z}`}
                {...mouseHoverEvent}
                {...mouseClickEvent({
                  text: displayName,
                  mapScale,
                  position: {
                    x: extract.position.x,
                    y: extract.position.z,
                  },
                  real2imagePos,
                })}
                opacity={active ? 1 : 0.1}
                listening={active}
              >
                <Image
                  id={`im-extract-image-${extract.id}-${extract.position.x}-${extract.position.z}`}
                  x={real2imagePos.x(extract.position.x) - (12 * iconScale) / mapScale}
                  y={real2imagePos.y(extract.position.z) - (20 * iconScale) / mapScale}
                  width={Math.max(minIconSize, (24 * iconScale) / mapScale)}
                  height={Math.max(minIconSize, (24 * iconScale) / mapScale)}
                  imageSrc={getIconCDN(`extract_${extract.faction}`)}
                />
                <Text
                  id={`im-extract-text-${extract.id}-${extract.position.x}-${extract.position.z}`}
                  x={real2imagePos.x(extract.position.x)}
                  y={real2imagePos.y(extract.position.z)}
                  fontFamily="JinBuTi"
                  text={displayName}
                  fontSize={Math.max(minFontSize, (12 * textScale) / mapScale)}
                  fill={extractColorMap[extract.faction]}
                  width={(600 * textScale) / mapScale}
                  offsetX={(300 * textScale) / mapScale}
                  align="center"
                  shadowColor="#000000"
                  shadowBlur={(12 * textScale) / mapScale}
                  offsetY={(-6 * textScale) / mapScale}
                  listening={false}
                />
              </Group>
            );
          } else {
            return null;
          }
        })}
      </Group>
    );
  } else {
    return null;
  }
};

Index.displayName = 'Extracts';
export default React.memo(Index);
