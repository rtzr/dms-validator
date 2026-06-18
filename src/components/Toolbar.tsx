import { SAMPLES } from "../lib/samples";
import { Tooltip } from "./Tooltip";

const DMS_VERSIONS = ["3.5.4", "3.5.2", "3.5.1", "3.5.0", "3.4.7", "3.4.6"];

const VERSION_TIP =
  "버전 게이팅 기준 DMS 엔진 버전입니다. 일부 기능은 최소 버전이 필요하며, 선택값보다 높은 버전을 요구하는 기능을 쓰면 경고로 표시됩니다.\n" +
  "• 데이터 마스킹(data-masking-*): 3.5.4+\n" +
  "• 필터 혼합(긍정+부정) 연산자: 3.5.0+\n" +
  "• Kafka 멀티토픽(kafka-target-topic): 3.4.6+";

interface Props {
  fileName: string | null;
  dirty: boolean;
  tauri: boolean;
  live: boolean;
  version: string;
  onOpen: () => void;
  onSave: () => void;
  onLoadSample: (content: string) => void;
  onValidate: () => void;
  onToggleLive: (v: boolean) => void;
  onVersionChange: (v: string) => void;
}

export function Toolbar(props: Props) {
  const {
    fileName,
    dirty,
    tauri,
    live,
    version,
    onOpen,
    onSave,
    onLoadSample,
    onValidate,
    onToggleLive,
    onVersionChange,
  } = props;

  const name = fileName ? fileName.split(/[/\\]/).pop() : "untitled.json";

  return (
    <div className="toolbar">
      <div className="toolbar-group">
        <button onClick={onOpen} disabled={!tauri} title={tauri ? "" : "Tauri 앱에서만 사용 가능"}>
          열기
        </button>
        <button onClick={onSave} disabled={!tauri}>
          저장
        </button>
        <select
          className="sample-select"
          defaultValue=""
          onChange={(e) => {
            const s = SAMPLES.find((x) => x.id === e.target.value);
            if (s) onLoadSample(s.content);
            e.target.value = "";
          }}
        >
          <option value="" disabled>
            샘플 ▾
          </option>
          {SAMPLES.map((s) => (
            <option key={s.id} value={s.id}>
              {s.label}
            </option>
          ))}
        </select>
      </div>

      <div className="toolbar-file">
        <span className="file-name">{name}</span>
        {dirty && <span className="dirty-dot" title="저장되지 않은 변경">●</span>}
      </div>

      <div className="toolbar-group">
        <label className="live-toggle" title="편집 시 자동으로 검증합니다 (입력 후 약 0.4초 디바운스). 끄면 [검증] 버튼으로 수동 실행합니다.">
          <input
            type="checkbox"
            checked={live}
            onChange={(e) => onToggleLive(e.target.checked)}
          />
          실시간 검증
        </label>
        <button onClick={onValidate} className="primary" title="현재 문서를 지금 검증합니다">
          검증
        </button>
        <label className="version-select">
          DMS
          <select value={version} onChange={(e) => onVersionChange(e.target.value)}>
            {DMS_VERSIONS.map((v) => (
              <option key={v} value={v}>
                {v}
              </option>
            ))}
          </select>
          <Tooltip side="bottom" text={VERSION_TIP}>
            <span className="help-icon" aria-label="DMS 버전 게이팅 설명">
              ⓘ
            </span>
          </Tooltip>
        </label>
      </div>
    </div>
  );
}
